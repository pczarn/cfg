use std::ops::Deref;

use crate::history::{HistoryGraph, HistoryId, HistoryNode};
use crate::precedence::PrecedencedRuleBuilder;
use crate::prelude::*;
use crate::rule::builder::RuleBuilder;
use crate::rule::GrammarRule;
use crate::symbol::source::SymbolContainer;

/// Trait for rule and symbol containers.
pub trait RuleContainer: Sized {
    /// Returns an immutable reference to the grammar's symbol source.
    fn sym_source(&self) -> &SymbolSource;

    /// Returns a mutable reference to the grammar's symbol source.
    fn sym_source_mut(&mut self) -> &mut SymbolSource;

    /// Returns generated symbols.
    fn sym<T>(&mut self) -> T
    where
        T: SymbolContainer,
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
    where
        F: FnMut(Symbol, &[Symbol], HistoryId) -> bool;

    /// Inserts a rule with `lhs` and `rhs` on its LHS and RHS. The rule carries `history`.
    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: HistoryId);

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
}

impl<'a, D> RuleContainer for &'a mut D
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
        F: FnMut(Symbol, &[Symbol], HistoryId) -> bool,
    {
        (**self).retain(f);
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: HistoryId) {
        (**self).add_rule(lhs, rhs, history);
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        (**self).add_history_node(node)
    }

    fn history_graph(&self) -> &HistoryGraph {
        (**self).history_graph()
    }
}

// Traits for working around the lack of higher-order type constructors, more commonly known as HKT
// or HKP.

/// This trait is currently needed to make the associated `Rules` iterator generic over a lifetime
/// parameter.
pub trait RuleContainerRef<'a>: Deref + Sized
where
    Self::Target: RuleContainer + Default,
{
    /// Immutable reference to a rule.
    type RuleRef: GrammarRule + Copy + 'a;
    /// Iterator over immutable references to the grammar's rules.
    type Rules: Iterator<Item = Self::RuleRef>;
    /// Returns an iterator over immutable references to the grammar's rules.
    fn rules(self) -> Self::Rules;

    /// Reverses the grammar.
    fn reverse(self) -> Self::Target
    where
        Self::Target: Default,
    {
        let mut new_grammar: Self::Target = Default::default();
        for _ in 0..self.sym_source().num_syms() {
            let _: Symbol = new_grammar.sym();
        }

        for node in self.history_graph().iter() {
            new_grammar.add_history_node(node.clone());
        }

        for rule in self.rules() {
            let mut rhs = rule.rhs().iter().cloned().collect::<Vec<_>>();
            rhs.reverse();
            new_grammar.add_rule(rule.lhs(), &rhs[..], rule.history_id());
        }
        new_grammar
    }
}

/// Allows access to a ContextFreeRef through mutable references.
pub trait RuleContainerMut<'a>: Deref
where
    Self::Target: RuleContainer + Default + 'a,
    &'a Self::Target: RuleContainerRef<'a, Target = Self::Target>,
{
}
