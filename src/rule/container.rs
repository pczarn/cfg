//! Abstraction for collections of rules.

use rule::terminal_set::TerminalSet;
use symbol::SymbolSource;

/// Trait for rule and symbol containers.
pub trait RuleContainer: SymbolSource {
    /// The type of history carried with the rule.
    type History;
    /// The type of information about whether symbols are terminal or nonterminal.
    type TerminalSet: TerminalSet;

    /// Retains only the rules specified by the predicate.
    ///
    /// In other words, removes all rules `rule` for which `f(&rule)` returns false.
    fn retain<F>(&mut self, f: F) where
            F: FnMut(Self::Symbol, &[Self::Symbol], &Self::History) -> bool;
    /// Inserts a rule with `lhs` and `rhs` on its LHS and RHS. The rule carries `history`.
    fn add_rule(&mut self, lhs: Self::Symbol,
                           rhs: &[Self::Symbol],
                           history: Self::History);
    /// Gathers information about whether symbols are terminal or nonterminal.
    ///
    /// Constructs a data structure in O(n) time.
    fn terminal_set(&self) -> Self::TerminalSet;
}

impl<'a, D> RuleContainer for &'a mut D where D: RuleContainer {
    type History = D::History;
    type TerminalSet = D::TerminalSet;

    fn retain<F>(&mut self, f: F) where
                F: FnMut(Self::Symbol, &[Self::Symbol], &Self::History) -> bool {
        (**self).retain(f);
    }

    fn add_rule(&mut self, lhs: Self::Symbol,
                           rhs: &[Self::Symbol],
                           history: Self::History) {
        (**self).add_rule(lhs, rhs, history);
    }

    fn terminal_set(&self) -> Self::TerminalSet {
        (**self).terminal_set()
    }
}
