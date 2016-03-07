//! Abstraction for collections of rules.

use symbol::{Symbol, SymbolSource};
use symbol::source::SymbolContainer;

/// Trait for rule and symbol containers.
pub trait RuleContainer {
    /// The type of history carried with the rule.
    type History;

    /// Returns an immutable reference to the grammar's symbol source.
    fn sym_source(&self) -> &SymbolSource;

    /// Returns a mutable reference to the grammar's symbol source.
    fn sym_source_mut(&mut self) -> &mut SymbolSource;

    /// Returns generated symbols.
    fn sym<T>(&mut self) -> T
        where T: SymbolContainer
    {
        self.sym_source_mut().sym()
    }

    /// Generates a new unique symbol.
    fn next_sym(&mut self) -> Symbol {
        self.sym_source_mut().next_sym()
    }

    /// Returns the number of symbols in use.
    fn num_syms(&self) -> usize {
        self.sym_source().num_syms()
    }

    /// Retains only the rules specified by the predicate.
    ///
    /// In other words, removes all rules `rule` for which `f(&rule)` returns false.
    fn retain<F>(&mut self, f: F)
        where F: FnMut(Symbol, &[Symbol], &Self::History) -> bool;
    /// Inserts a rule with `lhs` and `rhs` on its LHS and RHS. The rule carries `history`.
    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: Self::History);
}

impl<'a, D> RuleContainer for &'a mut D where D: RuleContainer
{
    type History = D::History;

    fn sym_source(&self) -> &SymbolSource {
        (**self).sym_source()
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        (**self).sym_source_mut()
    }

    fn retain<F>(&mut self, f: F)
        where F: FnMut(Symbol, &[Symbol], &Self::History) -> bool
    {
        (**self).retain(f);
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: Self::History) {
        (**self).add_rule(lhs, rhs, history);
    }
}
