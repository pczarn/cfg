//! Informs whether symbols are terminal or nonterminal.

use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef};
use rule::GrammarRule;
use symbol::Symbol;

/// Informs whether symbols are terminal or nonterminal.
pub trait TerminalSet {
    /// Checks whether a given symbol is in this set.
    fn has_sym(&self, sym: Symbol) -> bool;
    /// Converts into a bit vector.
    fn into_bit_vec(self) -> BitVec;
}

/// Information about whether symbols are terminal or nonterminal, in the form of a bit vector.
pub struct TerminalBitSet {
    bit_vec: BitVec,
}

impl TerminalSet for TerminalBitSet {
    fn has_sym(&self, sym: Symbol) -> bool {
        self.bit_vec[sym.into()]
    }

    fn into_bit_vec(self) -> BitVec {
        self.bit_vec
    }
}

impl TerminalBitSet {
    /// Constructs a `TerminalBitSet`.
    pub fn new<'a, G>(grammar: &'a G) -> Self
        where G: ContextFree,
              &'a G: ContextFreeRef<'a, Target = G>,
    {
        let mut bit_vec = BitVec::from_elem(grammar.num_syms(), true);

        for rule in grammar.rules() {
            if !rule.rhs().is_empty() {
                bit_vec.set(rule.lhs().into(), false);
            }
        }

        TerminalBitSet {
            bit_vec: bit_vec,
        }
    }
}
