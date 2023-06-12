use std::mem;
use std::slice;

use crate::binarized::BinarizedCfg;
use crate::history::{HistoryGraph, HistoryId, HistoryNode};
use crate::prelude::*;
use crate::rule::{GrammarRule, Rule};
use crate::sequence::builder::SequenceRuleBuilder;
use crate::sequence::rewrite::SequencesToProductions;
use crate::sequence::Sequence;
use crate::symbol::source::SymbolContainer;

/// Basic representation of context-free grammars.
#[derive(Clone)]
pub struct Cfg {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<Rule>,
    /// The array of sequence rules.
    sequence_rules: Vec<Sequence>,
    /// History container.
    history_graph: HistoryGraph,
}

impl Default for Cfg {
    fn default() -> Self {
        Self::with_sym_source(SymbolSource::new())
    }
}

impl Cfg {
    /// Creates an empty context-free grammar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty context-free grammar with the given symbol source.
    pub fn with_sym_source(sym_source: SymbolSource) -> Self {
        Cfg {
            sym_source: sym_source,
            rules: vec![],
            sequence_rules: vec![],
            history_graph: HistoryGraph::new(),
        }
    }
}

impl Cfg {
    /// Returns generated symbols.
    pub fn sym<T>(&mut self) -> T
    where
        T: SymbolContainer,
    {
        self.sym_source_mut().sym()
    }

    /// Generates a new unique symbol.
    pub fn next_sym(&mut self) -> Symbol {
        self.sym_source_mut().next_sym()
    }

    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.sym_source().num_syms()
    }

    /// Starts building a sequence rule.
    pub fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<&mut Vec<Sequence>> {
        SequenceRuleBuilder::new(&mut self.sequence_rules).sequence(lhs)
    }

    /// Returns sequence rules.
    pub fn sequence_rules(&self) -> &[Sequence] {
        &self.sequence_rules
    }

    /// Forces a rewrite of sequence rules into grammar rules.
    pub fn rewrite_sequences(&mut self) {
        let sequence_rules = mem::replace(&mut self.sequence_rules, vec![]);
        SequencesToProductions::rewrite_sequences(&sequence_rules[..], self);
    }

    /// Returns a binarized grammar which is weakly equivalent to this grammar.
    pub fn binarize<'a>(&'a self) -> BinarizedCfg
    where
        &'a Self: RuleContainerRef<'a, Target = Self>,
    {
        let mut grammar = BinarizedCfg::from_context_free(self);
        SequencesToProductions::rewrite_sequences(&self.sequence_rules[..], &mut grammar);
        grammar
    }
}

impl RuleContainer for Cfg {
    fn history_graph(&self) -> &HistoryGraph {
        &self.history_graph
    }

    fn sym_source(&self) -> &SymbolSource {
        &self.sym_source
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        &mut self.sym_source
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(Symbol, &[Symbol], HistoryId) -> bool,
    {
        self.rules
            .retain(|rule| f(rule.lhs(), rule.rhs(), rule.history_id()));
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: HistoryId) {
        self.rules.push(Rule::new(lhs, rhs.to_vec(), history));
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.history_graph.next_id();
        self.history_graph.push(node);
        result
    }
}

impl<'a> RuleContainerRef<'a> for &'a Cfg {
    type RuleRef = <Self::Rules as Iterator>::Item;
    type Rules = slice::Iter<'a, Rule>;

    fn rules(self) -> Self::Rules {
        self.rules.iter()
    }
}

impl<'a> RuleContainerMut<'a> for &'a mut Cfg {}
