use crate::history::{HistoryGraph, HistoryId, HistoryNode};
use crate::local_prelude::*;
use crate::precedenced_rule::PrecedencedRuleBuilder;
use crate::rule::builder::RuleBuilder;
use crate::rule::RuleRef;

/// Trait for rule and symbol containers.
pub trait RuleContainer: Sized {
    /// Returns an immutable reference to the grammar's symbol source.
    fn sym_source(&self) -> &SymbolSource;

    /// Returns a mutable reference to the grammar's symbol source.
    fn sym_source_mut(&mut self) -> &mut SymbolSource;

    /// Returns generated symbols.
    fn sym<const N: usize>(&mut self) -> [Symbol; N] {
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
    where
        F: FnMut(RuleRef) -> bool;

    /// Inserts a rule with `lhs` and `rhs` on its LHS and RHS. The rule carries `history`.
    fn add_rule(&mut self, rule_ref: RuleRef);

    /// Starts building a new rule.
    fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut Self> {
        RuleBuilder::new(self).rule(lhs)
    }

    /// Starts building a new precedenced rule.
    fn precedenced_rule(&mut self, lhs: Symbol) -> PrecedencedRuleBuilder<&mut Self> {
        PrecedencedRuleBuilder::new(self, lhs)
    }

    fn history_graph(&self) -> &HistoryGraph;

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId;

    fn rules<'a>(&'a self) -> impl Iterator<Item = RuleRef<'a>>
    where
        Self: 'a;
    // fn rules<'a>(&'a self) -> Self::Rules<'a>;

    /// Reverses the grammar.
    fn reverse(&self) -> Self
    where
        Self: Default,
    {
        let mut new_grammar: Self = Default::default();
        for _ in 0..self.sym_source().num_syms() {
            let [_] = new_grammar.sym();
        }

        for node in self.history_graph().iter() {
            new_grammar.add_history_node(node.clone());
        }

        for rule in self.rules() {
            let mut rhs = rule.rhs.iter().cloned().collect::<Vec<_>>();
            rhs.reverse();
            let rhs = &rhs[..];
            new_grammar.add_rule(RuleRef { rhs, ..rule });
        }
        new_grammar
    }
}

impl<'r, D> RuleContainer for &'r mut D
where
    D: RuleContainer,
{
    fn sym_source(&self) -> &SymbolSource {
        (**self).sym_source()
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        (**self).sym_source_mut()
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(RuleRef) -> bool,
    {
        (**self).retain(f);
    }

    fn add_rule(&mut self, rule_ref: RuleRef) {
        (**self).add_rule(rule_ref);
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        (**self).add_history_node(node)
    }

    fn history_graph(&self) -> &HistoryGraph {
        (**self).history_graph()
    }

    fn rules<'a>(&'a self) -> impl Iterator<Item = RuleRef<'a>>
    where
        Self: 'a,
    {
        // fn rules<'b>(&'b self) -> Self::Rules<'b> {
        (**self).rules()
    }

    /// Reverses the grammar.
    fn reverse(&self) -> Self
    where
        Self: Default,
    {
        let new_grammar: Self = Default::default();
        for _ in 0..self.sym_source().num_syms() {
            let [_] = new_grammar.sym();
        }

        for node in self.history_graph().iter() {
            new_grammar.add_history_node(node.clone());
        }

        for rule in self.rules() {
            let mut rhs = rule.rhs.iter().cloned().collect::<Vec<_>>();
            rhs.reverse();
            let rhs = &rhs[..];
            new_grammar.add_rule(RuleRef { rhs, ..rule });
        }
        new_grammar
    }
}
