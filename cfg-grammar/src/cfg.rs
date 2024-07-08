use crate::history::{HistoryGraph, HistoryId, HistoryNode};
use crate::rule::cfg_rule::CfgRule;
use crate::rule::RuleRef;
use crate::BinarizedCfg;
use crate::{local_prelude::*, AsRuleRef};

/// Basic representation of context-free grammars.
#[derive(Clone)]
pub struct Cfg {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<CfgRule>,
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
            history_graph: HistoryGraph::new(),
        }
    }
}

impl Cfg {
    /// Returns generated symbols.
    pub fn sym<const N: usize>(&mut self) -> [Symbol; N] {
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

    /// Returns a binarized grammar which is weakly equivalent to this grammar.
    pub fn binarize(&self) -> BinarizedCfg {
        BinarizedCfg::from_context_free(self)
    }
}

impl RuleContainer for Cfg {
    fn rules<'a>(&'a self) -> impl Iterator<Item = RuleRef<'a>>
    where
        Self: 'a,
    {
        self.rules.iter().map(|rule| rule.as_rule_ref())
    }

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
        F: FnMut(RuleRef) -> bool,
    {
        self.rules.retain(|rule| f(rule.as_rule_ref()));
    }

    fn add_rule(&mut self, rule_ref: RuleRef) {
        self.rules.push(CfgRule {
            lhs: rule_ref.lhs,
            rhs: rule_ref.rhs.to_vec(),
            history_id: rule_ref.history_id,
        });
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.history_graph.next_id();
        self.history_graph.push(node);
        result
    }
}
