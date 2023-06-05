//! Source

use symbol::repr::{SymbolRepr, FIRST_ID};
use symbol::Symbol;

/// A source of numeric symbols.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Default)]
pub struct SymbolSource {
    next_id: SymbolRepr,
}

impl SymbolSource {
    /// Creates a source of numeric symbols with an empty symbol space.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns generated symbols.
    pub fn sym<T>(&mut self) -> T
    where
        T: SymbolContainer,
    {
        T::generate(self)
    }
    /// Generates a new unique symbol.
    pub fn next_sym(&mut self) -> Symbol {
        let ret = Symbol::from(self.next_id);
        self.next_id = self.next_id.saturating_add(1 + FIRST_ID) - FIRST_ID;
        ret
    }
    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.next_id as usize
    }
    /// Returns an iterator that generates symbols.
    pub fn generate(&mut self) -> Generate {
        Generate { source: self }
    }
}

/// Trait used to generate symbols.
pub trait SymbolContainer {
    /// Generates symbols.
    fn generate(source: &mut SymbolSource) -> Self;
}

macro_rules! impl_generate {
    (Symbol, next_sym, $($T:ident, $next_sym:ident,)*) => (
        #[allow(unused_parens)]
        impl SymbolContainer for ( Symbol $(, $T)* ) {
            fn generate(source: &mut SymbolSource) -> Self {
                (source.next_sym()
                 $(, source.$next_sym())*)
            }
        }
        impl_generate!($($T, $next_sym,)*);
    );
    // base case
    () => {}
}

impl_generate!(
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
    Symbol, next_sym, Symbol, next_sym, Symbol, next_sym, Symbol, next_sym,
);

/// Iterator for generating symbols.
pub struct Generate<'a> {
    source: &'a mut SymbolSource,
}

impl<'a> Iterator for Generate<'a> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sym())
    }
}
