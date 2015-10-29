//! Symbol types can be used to parameterize grammars.

use core::nonzero::NonZero;
use std::convert::{From, Into};
use std::hash::Hash;

/// A numeric symbol type.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct NumericSymbol(NonZero<NumericSymbolRepr>);

type NumericSymbolRepr = u32;

/// A type that can represent symbols in a context-free grammar. Symbols are distinguished by their
/// IDs.
pub trait GrammarSymbol: From<u64> + Into<u64> + Eq + Ord + Clone + Hash + Copy {
    /// Cast the symbol's ID to `usize`.
    fn usize(&self) -> usize {
        let id: u64 = (*self).into();
        id as usize
    }
}

impl From<u64> for NumericSymbol {
    fn from(id: u64) -> Self {
        unsafe { NumericSymbol(NonZero::new(id as NumericSymbolRepr)) }
    }
}

impl Into<u64> for NumericSymbol {
    fn into(self) -> u64 {
        *self.0 as u64
    }
}

impl GrammarSymbol for NumericSymbol {}

/// A source of symbols.
pub trait SymbolSource {
    /// The type of symbols generated by this source.
    type Symbol: GrammarSymbol;

    /// Generates a new unique symbol.
    fn next_sym(&mut self, terminal: bool) -> Self::Symbol;
    /// Marks a symbol as nonterminal. Used each time a rule with arbitrary LHS is added
    /// to the grammar.
    fn mark_as_nonterminal(&mut self, sym: Self::Symbol);
    /// Returns the start symbol.
    fn start_sym(&self) -> Self::Symbol;
    /// Returns the number of symbols in use.
    fn num_syms(&self) -> usize;
    /// Returns an iterator that generates terminal symbols.
    fn terminals(&mut self) -> Terminals<&mut Self> {
        Terminals { source: self }
    }
    /// Returns an iterator that generates nonterminal symbols.
    fn nonterminals(&mut self) -> Nonterminals<&mut Self> {
        Nonterminals { source: self }
    }
    /// Returns generated terminal symbols.
    fn sym<T>(&mut self) -> T where Self: Sized, T: SymbolContainer<Self::Symbol> {
        T::generate(self)
    }
}

/// A source of symbols that tracks whether a symbol is terminal or nonterminal.
pub trait TerminalSymbolSet: SymbolSource {
    /// Checks whether a symbol is terminal.
    fn is_terminal(&self, sym: Self::Symbol) -> bool;
}

impl<'a, S> SymbolSource for &'a mut S where S: SymbolSource {
    type Symbol = S::Symbol;

    fn next_sym(&mut self, terminal: bool) -> Self::Symbol { (**self).next_sym(terminal) }
    fn mark_as_nonterminal(&mut self, sym: Self::Symbol) { (**self).mark_as_nonterminal(sym) }
    fn num_syms(&self) -> usize { (**self).num_syms() }
    fn start_sym(&self) -> Self::Symbol { (**self).start_sym() }
}

impl<'a, S> TerminalSymbolSet for &'a mut S where S: TerminalSymbolSet {
    fn is_terminal(&self, sym: Self::Symbol) -> bool { (**self).is_terminal(sym) }
}

/// A source of numeric symbols.
#[derive(Clone, Debug)]
pub struct ConsecutiveSymbols {
    next_sym: NumericSymbolRepr,
}

/// Iterator for generating terminal symbols.
pub struct Terminals<S> {
    source: S,
}

/// Iterator for generating nonterminal symbols.
pub struct Nonterminals<S> {
    source: S,
}

/// The start symbol's ID.
const START_SYMBOL: u32 = 1;
/// The first usable symbol ID.
const FIRST_SYMBOL: u32 = 2;

impl ConsecutiveSymbols {
    /// Creates a source of numeric symbols with an empty symbol space.
    pub fn new() -> Self {
        ConsecutiveSymbols {
            next_sym: FIRST_SYMBOL,
        }
    }

    /// Returns generated terminal symbols.
    pub fn sym<T>(&mut self) -> T where T: SymbolContainer<NumericSymbol> {
        T::generate(self)
    }
}

impl SymbolSource for ConsecutiveSymbols {
    type Symbol = NumericSymbol;

    fn next_sym(&mut self, _terminal: bool) -> NumericSymbol {
        let ret = unsafe { NumericSymbol(NonZero::new(self.next_sym)) };
        self.next_sym = self.next_sym.saturating_add(1);
        ret
    }

    fn mark_as_nonterminal(&mut self, _sym: Self::Symbol) {
        // This information isn't stored.
    }

    fn start_sym(&self) -> NumericSymbol {
        unsafe {
            NumericSymbol(NonZero::new(START_SYMBOL))
        }
    }

    fn num_syms(&self) -> usize {
        self.next_sym as usize
    }
}

impl<S> Iterator for Terminals<S> where S: SymbolSource {
    type Item = S::Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sym(true))
    }
}

impl<S> Iterator for Nonterminals<S> where S: SymbolSource {
    type Item = S::Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.source.next_sym(false))
    }
}

/// Trait used to generate terminal symbols.
pub trait SymbolContainer<S: GrammarSymbol> {
    /// Generates 
    fn generate<Ss>(source: Ss) -> Self where Ss: SymbolSource<Symbol=S>;
}

macro_rules! impl_generate {
    (S $(, $t:ident)*) => (
        impl<S> SymbolContainer<S> for ( S $(, $t)* ) where S: GrammarSymbol {
            fn generate<Ss>(mut source: Ss) -> Self where Ss: SymbolSource<Symbol=S> {
                ({ let x: S = source.next_sym(true); x }
                 $(, { let x: $t = source.next_sym(true); x })*)
            }
        }
        impl_generate!($($t),*);
    );
    // base case
    () => {}
}

impl_generate!(S, S, S, S, S, S, S, S, S, S, S, S, S, S, S, S);
