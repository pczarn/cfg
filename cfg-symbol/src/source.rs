//! Source

use std::{borrow::Cow, iter, num::NonZeroU32, rc::Rc};

use crate::{symbol::SymbolPrimitive, *};

/// A source of numeric symbols.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Default)]
pub struct SymbolSource<T: SymbolPrimitive = NonZeroU32> {
    next_symbol: Symbol<T>,
    names: Vec<Option<Rc<String>>>,
}

#[macro_export]
macro_rules! syms {
    () => {
        $crate::SymbolSource::generate_fresh()  
    };
}

fn advance<T: SymbolPrimitive>(sym: &mut Symbol<T>) {
    let n: NonZeroU32 = sym.n.into();
    let x: T = n.saturating_add(1).try_into().ok().expect("unreachable");
    *sym = Symbol { n: x };
    debug_assert_ne!(x.into().get(), T::MAX, "ran out of Symbol space?");
}

impl<T: SymbolPrimitive> SymbolSource<T> {
    /// Creates a source of numeric symbols with an empty symbol space.
    pub fn new() -> Self {
        Self { next_symbol: Symbol::first(), names: vec![] }
    }
    /// Returns generated symbols.
    pub fn sym<'a, const N: usize>(&mut self) -> [Symbol<T>; N] {
        let mut result = [Symbol::first(); N];
        for dest in result.iter_mut() {
            *dest = self.next_sym(None);
        }
        self.names.extend([const { None }; N]);
        result
    }
    /// Returns generated symbols.
    pub fn with_names<'a, const N: usize, S: Into<Cow<'static, str>>>(&mut self, names: [Option<S>; N]) -> [Symbol<T>; N] {
        let mut result = [Symbol::first(); N];
        for (dest, name) in result.iter_mut().zip(names.into_iter()) {
            *dest = self.next_sym(name.map(|s| s.into()));
        }
        self.names.extend([const { None }; N]);
        result
    }
    /// Generates a new unique symbol.
    pub fn next_sym(&mut self, name: Option<Cow<str>>) -> Symbol<T> {
        let ret = self.next_symbol;
        advance(&mut self.next_symbol);
        self.names.push(name.map(|cow| Rc::new(cow.to_string())));
        ret
    }
    pub fn name_of(&self, sym: Symbol) -> Cow<str> {
        match self.names.get(sym.usize()) {
            Some(Some(name)) => {
                Cow::Borrowed(&name[..])
            }
            Some(None) | None => {
                Cow::Owned(format!("g{}", sym.usize()))
            }
        }
    }
    pub fn original_name_of(&self, sym: Symbol) -> Option<&str> {
        self.names.get(sym.usize()).map(|v| v.as_ref().map(|v| &v[..])).unwrap_or(None)
    }
    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.next_symbol.usize()
    }
    /// Returns an iterator that generates symbols.
    pub fn generate(&mut self) -> impl Iterator<Item = Symbol<T>> + use<'_, T> {
        Generate { source: self }
    }

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

    // Translates symbol names to new symbol IDs.
    pub fn remap_symbols<F>(&mut self, mut map: F)
    where
        F: FnMut(Symbol<T>) -> Symbol<T>,
    {
        let mut new_names = vec![];
        let mut new_source = SymbolSource::<T>::new();
        for (name, sym) in self.names.iter().zip(new_source.generate()) {
            let new_sym = map(sym).usize();
            new_names.extend(iter::repeat(None).take((new_sym + 1).saturating_sub(new_names.len())));
            new_names[new_sym] = name.clone();
        }
        self.names = new_names;
    }

    pub fn truncate(&mut self, new_len: usize) {
        assert_ne!(new_len, 0, "cannot truncate to zero length");
        assert!(new_len as u64 <= T::MAX as u64, "attempt to truncate to too high length");
        self.names.truncate(new_len);
        self.next_symbol = Symbol { n: NonZeroU32::new(new_len as u32).expect("unreachable").try_into().ok().expect("unreachable") };
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
