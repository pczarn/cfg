//! Abstraction for collections of rules.

use symbol::SymbolSource;

/// Trait for rule and symbol containers.
pub trait RuleContainer: SymbolSource {
    /// The type of history carried with the rule.
    type History;

    /// Retains only the rules specified by the predicate.
    ///
    /// In other words, removes all rules `rule` for which `f(&rule)` returns false.
    fn retain<F>(&mut self, f: F) where
            F: FnMut(Self::Symbol, &[Self::Symbol], &Self::History) -> bool;
    /// Inserts a rule with `lhs` and `rhs` on its LHS and RHS. The rule carries `history`.
    fn add_rule(&mut self, lhs: Self::Symbol,
                           rhs: &[Self::Symbol],
                           history: Self::History);
}

impl<'a, D> RuleContainer for &'a mut D where D: RuleContainer {
    type History = D::History;

    fn retain<F>(&mut self, f: F) where
                F: FnMut(Self::Symbol, &[Self::Symbol], &Self::History) -> bool {
        (**self).retain(f);
    }

    fn add_rule(&mut self, lhs: Self::Symbol,
                           rhs: &[Self::Symbol],
                           history: Self::History) {
        (**self).add_rule(lhs, rhs, history);
    }
}
