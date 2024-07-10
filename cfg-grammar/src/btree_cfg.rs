use std::collections::BTreeMap;

use crate::symbol::SymbolSource;
use crate::history::HistoryGraph;
use crate::rule::Rule;
use crate::sequence::Sequence;
use crate::{Cfg, GrammarRule, Symbol};

use super::rule_container::RuleContainer;

pub type RuleId = usize;

/// Basic representation of context-free grammars.
#[derive(Clone)]
pub struct BTreeCfg {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: BTreeMap<(Symbol, RuleId), BTreeRule>,
    /// History container.
    history_graph: HistoryGraph,
}

pub enum BTreeRule {
    Rule(Rule),
    Sequence(Sequence),
}

pub struct RuleRef<'a> {
    lhs: Symbol,
    rule: &'a BTreeRule,
}

impl<'a> GrammarRule for RuleRef<'a> {
    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        match self.rule {
            &BTreeRule::Rule(Rule {  }) => {
                
            }
        }
    }
}

impl RuleContainer for BTreeCfg {
    type Rule<'a> = RuleRef<'a>;
    type Rules<'a> = slice::Iter<'a, Rule>;

    fn rules<'a>(&'a self) -> Self::Rules<'a> {
        self.rules.iter().map(|(&key, val)| {
            BTreeRuleRef {

            }
        })
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
        F: FnMut(Symbol, &[Symbol], 
        ) -> bool,
    {
        self.rules
            .retain(|&(lhs, id), rule| f(lhs, rule.rhs(), rule.history_id()));
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history_id: HistoryId) {
        let next_id = self.rules.range((lhs, 0) .. (lhs, usize::MAX)).last().map(|(&(_, num), _val)| num + 1).unwrap_or(0);
        self.rules.insert((lhs, next_id), BTreeRule::Rule(Rule { rhs: rhs.to_vec(), history_id }));
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.history_graph.next_id();
        self.history_graph.push(node);
        result
    }
}
