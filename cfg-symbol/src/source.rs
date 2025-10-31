//! Our symbol sources. You can grab symbols from here.
//!
//! A symbol source is meant to track the number of
//! symbols that were generated, as well as their names
//! (optionally).

use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    iter,
    num::NonZeroU32,
    ops,
    rc::Rc,
};

use crate::*;

/// Wrapper for a string holding a symbol's name. Meant to be cheap to clone.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SymbolName {
    name: Rc<str>,
}

impl ops::Deref for SymbolName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.name[..]
    }
}

impl From<Cow<'_, str>> for SymbolName {
    fn from(value: Cow<'_, str>) -> Self {
        SymbolName {
            name: Rc::from(&*value),
        }
    }
}

impl<'a> From<&'a str> for SymbolName {
    fn from(value: &'a str) -> Self {
        SymbolName {
            name: Rc::from(value),
        }
    }
}

impl Borrow<str> for SymbolName {
    fn borrow(&self) -> &str {
        &self.name[..]
    }
}

/// A source of numeric symbols.
#[derive(miniserde::Serialize, miniserde::Deserialize, Clone, Debug, Default)]
pub struct SymbolSource<T: SymbolPrimitive = NonZeroU32> {
    next_symbol: Symbol<T>,
    names: Vec<Option<SymbolName>>,
}

/// Shorthand for `SymbolSource::generate_fresh`.
#[macro_export]
macro_rules! syms {
    () => {
        $crate::SymbolSource::generate_fresh()
    };
}

fn advance<T: SymbolPrimitive>(sym: &mut Symbol<T>) {
    let n: NonZeroU32 = sym.n.into();
    let x: T = n
        .saturating_add(1)
        .try_into()
        .ok()
        .expect("unreachable: could not convert +1");
    *sym = Symbol { n: x };
    debug_assert_ne!(x.into().get(), T::MAX, "ran out of Symbol space?");
}

impl<T: SymbolPrimitive> SymbolSource<T> {
    /// Creates a source of numeric symbols with an empty symbol space.
    pub fn new() -> Self {
        Self {
            next_symbol: Symbol::first(),
            names: vec![],
        }
    }
    /// Returns generated symbols.
    pub fn sym<'a, const N: usize>(&mut self) -> [Symbol<T>; N] {
        let mut result = [Symbol::first(); N];
        for dest in result.iter_mut() {
            *dest = self.next_sym(None);
        }
        // self.names.extend([const { None }; N]);
        result
    }
    /// Returns generated symbols.
    pub fn with_names<'a, const N: usize, S: Into<Cow<'static, str>>>(
        &mut self,
        names: [Option<S>; N],
    ) -> [Symbol<T>; N] {
        let mut result = [Symbol::first(); N];
        for (dest, name) in result.iter_mut().zip(names.into_iter()) {
            *dest = self.next_sym(name.map(|s| s.into()));
        }
        // self.names.extend([const { None }; N]);
        result
    }
    /// Generates a new unique symbol.
    pub fn next_sym(&mut self, name: Option<Cow<str>>) -> Symbol<T> {
        let ret = self.next_symbol;
        advance(&mut self.next_symbol);
        self.names.push(name.map(|cow| cow.into()));
        ret
    }

    /// Returns either the formatted name if the given `Symbol` is a gensym,
    /// or the `Symbol`'s exact name.
    ///
    /// Gensyms have no names. That's why we create a 'formatted' name
    /// with the letter `g` followed by the symbol's numeric value.
    pub fn name_of(&self, sym: Symbol) -> Cow<'_, str> {
        match self.names.get(sym.usize()) {
            Some(Some(name)) => Cow::Borrowed(&name.name[..]),
            Some(None) | None => Cow::Owned(format!("g{}", sym.usize())),
        }
    }

    /// Returns the exact name, or `None` if the `Symbol` has no name (i.e.
    /// is a gensym).
    pub fn original_name_of(&self, sym: Symbol) -> Option<&str> {
        self.names
            .get(sym.usize())
            .map(|v| v.as_ref().map(|v| &v.name[..]))
            .unwrap_or(None)
    }

    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.next_symbol.usize()
    }

    /// Returns an iterator that generates symbols.
    pub fn generate(&mut self) -> impl Iterator<Item = Symbol<T>> + use<'_, T> {
        Generate { source: self }
    }

    /// Iterator over all possible `Symbol`s, in order, starting with the lowest
    /// numeric value.
    pub fn generate_fresh() -> impl Iterator<Item = Symbol<T>> {
        struct Unfolder<T: SymbolPrimitive>(Symbol<T>);

        impl<T: SymbolPrimitive> Iterator for Unfolder<T> {
            type Item = Symbol<T>;
            fn next(&mut self) -> Option<Symbol<T>> {
                let result = self.0;
                advance(&mut self.0);
                Some(result)
            }
        }

        Unfolder(Symbol::first())
    }

    /// Translates symbol names according to the given mapping
    /// beween `Symbol`s.
    pub fn remap_symbols<F>(&mut self, mut map: F)
    where
        F: FnMut(Symbol<T>) -> Symbol<T>,
    {
        let mut new_names = vec![];
        let mut new_source = SymbolSource::<T>::new();
        for (name, sym) in self.names.iter().zip(new_source.generate()) {
            let new_sym = map(sym).usize();
            new_names
                .extend(iter::repeat(None).take((new_sym + 1).saturating_sub(new_names.len())));
            new_names[new_sym] = name.clone();
        }
        self.names = new_names;
    }

    /// Removes all the symbols with numeric value equal or higher than
    /// the argument.
    ///
    /// # Panics
    ///
    /// Panics unless `new_len` is within `1 ..= T::MAX`.
    pub fn truncate(&mut self, new_len: usize) {
        assert_ne!(new_len, 0, "cannot truncate to zero length");
        assert!(
            new_len as u64 <= T::MAX as u64,
            "attempt to truncate to a length that exceeds the limit"
        );
        self.names.truncate(new_len);
        self.next_symbol = Symbol {
            n: NonZeroU32::new(new_len as u32 + 1)
                .expect("cannot truncate to this length")
                .try_into()
                .ok()
                .expect("cannot truncate to this length (2)"),
        };
    }

    /// Returns the list of `Symbol` names.
    ///
    /// # Performance
    ///
    /// Internally, bumps the reference count for every existing `SymbolName`
    /// and copies the pointers to a newly allocated a new `Vec`,
    pub fn names(&self) -> Vec<Option<SymbolName>> {
        self.names.clone()
    }

    /// Creates a `HashMap` where you can access a `Symbol`
    /// through its name.
    pub fn name_map(&self) -> HashMap<SymbolName, Symbol> {
        self.names
            .iter()
            .zip(SymbolSource::generate_fresh())
            .filter_map(|(opt, i)| opt.clone().map(|name| (name, i)))
            .collect::<HashMap<_, _>>()
    }
}

/// Iterator for generating symbols.
struct Generate<'a, T: SymbolPrimitive> {
    source: &'a mut SymbolSource<T>,
}

impl<'a, T: SymbolPrimitive> Iterator for Generate<'a, T> {
    type Item = Symbol<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sym(None))
    }
}

mod miniserde_impls {
    use super::SymbolName;
    use miniserde::de::{Deserialize, Visitor};
    use miniserde::{Result, make_place};
    use miniserde::{Serialize, de, ser};
    use std::rc::Rc;

    make_place!(Place);

    impl Visitor for Place<SymbolName> {
        fn string(&mut self, s: &str) -> Result<()> {
            self.out = Some(SymbolName { name: Rc::from(s) });
            Ok(())
        }
    }

    impl Deserialize for SymbolName {
        fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor {
            Place::new(out)
        }
    }

    impl Serialize for SymbolName {
        fn begin(&self) -> ser::Fragment<'_> {
            ser::Fragment::Str(self.name.to_string().into())
        }
    }
}
