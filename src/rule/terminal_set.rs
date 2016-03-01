//! Informs whether symbols are terminal or nonterminal.

use std::marker::PhantomData;

use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef};
use rule::GrammarRule;
use symbol::GrammarSymbol;

/// Informs whether symbols are terminal or nonterminal.
pub trait TerminalSet {
    /// Type of symbols.
    type Symbol: GrammarSymbol;

    /// Checks whether a given symbol is in this set.
    fn has_sym(&self, sym: Self::Symbol) -> bool;
    /// Converts into a bit vector.
    fn into_bit_vec(self) -> BitVec;
}

/// Information about whether symbols are terminal or nonterminal, in the form of a bit vector.
pub struct TerminalBitSet<S> {
    bit_vec: BitVec,
    marker: PhantomData<S>,
}

impl<S> TerminalSet for TerminalBitSet<S> where S: GrammarSymbol
{
    type Symbol = S;

    fn has_sym(&self, sym: S) -> bool {
        self.bit_vec[sym.usize()]
    }

    fn into_bit_vec(self) -> BitVec {
        self.bit_vec
    }
}

impl<S> TerminalBitSet<S> {
    /// Constructs a `TerminalBitSet`.
    pub fn new<'a, G>(grammar: &'a G) -> Self
        where G: ContextFree<Symbol = S>,
              &'a G: ContextFreeRef<'a, Target = G>,
              S: GrammarSymbol
    {
        let mut bit_vec = BitVec::from_elem(grammar.num_syms(), true);

        for rule in grammar.rules() {
            if !rule.rhs().is_empty() {
                bit_vec.set(rule.lhs().usize(), false);
            }
        }

        TerminalBitSet {
            bit_vec: bit_vec,
            marker: PhantomData,
        }
    }
}
