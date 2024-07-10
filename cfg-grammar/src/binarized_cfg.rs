//! Binarized rules are rules that have at most two symbols on the right-hand side.
//! A binarized grammar contains only such rules.

use std::cmp::{self, Ord, Ordering};
use std::iter;
use std::mem;

use bit_vec::BitVec;

use crate::history::node::{
    BinarizedRhsSubset::*, HistoryNodeBinarize, HistoryNodeEliminateNulling,
};
use crate::history::{HistoryGraph, HistoryId, HistoryNode};
use crate::local_prelude::*;
use crate::rhs_closure::RhsClosure;
use crate::rule::{AsRuleRef, RuleRef};

use self::BinarizedRuleRhs::*;

/// Representation for grammars where right-hand sides of all rules have at most two symbols.
#[derive(Clone)]
pub struct BinarizedCfg {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<BinarizedRule>,
    /// The set of history values carried with the grammar's nulling rules, or empty if the grammar
    /// has no nulling rules.
    nulling: Vec<Option<HistoryId>>,
    /// History graph.
    history_graph: HistoryGraph,
}

/// Compact representation of a binarized rule.
#[derive(Copy, Clone)]
pub struct BinarizedRule {
    lhs: Symbol,
    rhs: BinarizedRuleRhs,
    history_id: HistoryId,
}

/// Compact representation of a binarized rule's RHS.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum BinarizedRuleRhs {
    /// RHS with one symbol.
    One([Symbol; 1]),
    /// RHS with two symbols.
    Two([Symbol; 2]),
}

impl Default for BinarizedCfg {
    fn default() -> Self {
        Self::with_sym_source(SymbolSource::new())
    }
}

impl BinarizedCfg {
    /// Creates a BinarizedCfg.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty BinarizedCfg with the given symbol source.
    pub fn with_sym_source(sym_source: SymbolSource) -> Self {
        BinarizedCfg {
            sym_source: sym_source,
            rules: vec![],
            nulling: vec![],
            history_graph: HistoryGraph::new(),
        }
    }

    /// Creates a BinarizedCfg by binarizing a context-free grammar.
    pub fn from_context_free<G>(this: &G) -> BinarizedCfg
    where
        G: RuleContainer + Default,
    {
        let mut new_rule_count = 0;
        // Calculate rule vec capacity.
        for rule in this.rules() {
            let rule_ref = rule.as_rule_ref();
            if !rule_ref.rhs.is_empty() {
                new_rule_count += 1.max(rule_ref.rhs.len() - 1);
            }
        }
        // Create a new grammar.
        let mut grammar = BinarizedCfg::with_sym_source(this.sym_source().clone());
        grammar.history_graph = this.history_graph().clone();
        grammar.rules = Vec::with_capacity(new_rule_count);
        // Insert all rules from one grammar into the other.
        for rule in this.rules() {
            grammar.add_rule(rule);
        }

        grammar
    }

    /// Sorts the rule array.
    pub fn sort(&mut self) {
        self.rules.sort();
    }

    /// Sorts the rule array in place, using the argument to compare elements.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&BinarizedRule, &BinarizedRule) -> Ordering,
    {
        self.rules.sort_by(compare);
    }

    /// Removes consecutive duplicate rules.
    pub fn dedup(&mut self) {
        self.rules.dedup();
    }

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

    /// Eliminates all rules of the form `A ::= epsilon`.
    ///
    /// In other words, this splits off the set of nulling rules.
    ///
    /// The language represented by the grammar is preserved, except for the possible lack of
    /// the empty string. Unproductive rules aren't preserved.
    pub fn eliminate_nulling_rules(&mut self) -> BinarizedCfg {
        let mut nulling_grammar = BinarizedCfg::with_sym_source(self.sym_source.clone());

        if self.nulling.iter().any(|h| h.is_some()) {
            let mut nulling = mem::replace(&mut self.nulling, vec![]);
            let nulling_len = nulling.len();
            nulling.extend(iter::repeat(None).take(self.sym_source().num_syms() - nulling_len));
            let mut nullable: BitVec = nulling.iter().map(|n| n.is_some()).collect();
            let mut productive: BitVec = BitVec::from_elem(self.sym_source().num_syms(), true);
            // All rules of the form `A ::= ε` go to the nulling grammar.
            nulling_grammar.nulling = nulling;
            RhsClosure::new(self).rhs_closure(&mut nullable);
            // Add new rules.
            let mut rewritten_rules = Vec::new();
            for rule in self.rules() {
                let left_nullable = nullable[rule.rhs[0].into()];
                let right_nullable = rule.rhs.get(1).map_or(true, |s| nullable[s.usize()]);
                if left_nullable && right_nullable {
                    // The rule is copied to the nulling grammar.
                    let prev = rule.history_id;
                    let history_id = nulling_grammar.add_history_node(
                        HistoryNodeEliminateNulling {
                            prev,
                            rhs0: rule.rhs[0],
                            rhs1: rule.rhs.get(1).cloned(),
                            which: All,
                        }
                        .into(),
                    );
                    nulling_grammar.add_rule(RuleRef { history_id, ..rule });
                }
                if rule.as_rule_ref().rhs.len() == 2 {
                    if left_nullable {
                        let history = HistoryNodeEliminateNulling {
                            prev: rule.history_id,
                            rhs0: rule.rhs[0],
                            rhs1: rule.rhs.get(1).cloned(),
                            which: Left,
                        };
                        let rhs = &rule.rhs[1..2];
                        rewritten_rules
                            .push((BinarizedRule::new(RuleRef { rhs, ..rule }), history));
                    }
                    if right_nullable {
                        let history = HistoryNodeEliminateNulling {
                            prev: rule.history_id,
                            rhs0: rule.rhs[0],
                            rhs1: rule.rhs.get(1).cloned(),
                            which: Right,
                        };
                        let rhs = &rule.rhs[0..1];
                        rewritten_rules
                            .push((BinarizedRule::new(RuleRef { rhs, ..rule }), history));
                    }
                }
            }
            self.rules.extend(
                rewritten_rules
                    .into_iter()
                    .map(|(mut binarized_rule, history)| {
                        binarized_rule.history_id =
                            self.history_graph.add_history_node(history.into());
                        binarized_rule
                    }),
            );
            for rule in self.rules() {
                productive.set(rule.lhs.into(), true);
            }
            for rule in nulling_grammar.rules() {
                productive.set(rule.lhs.into(), false);
            }
            RhsClosure::new(self).rhs_closure(&mut productive);
            self.rules.retain(|rule| {
                // Retain the rule only if it's productive. We have to, in order to remove rules
                // that were made unproductive as a result of `A ::= epsilon` rule elimination.
                // Otherwise, some of our nonterminal symbols might be terminal.
                let left_productive = productive[rule.rhs0().into()];
                let right_productive = rule.rhs1().map_or(true, |s| productive[s.into()]);
                left_productive && right_productive
            });
        }

        nulling_grammar
    }
}

impl RuleContainer for BinarizedCfg {
    fn history_graph(&self) -> &HistoryGraph {
        &self.history_graph
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.history_graph.next_id();
        self.history_graph.push(node);
        result
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

    fn add_rule(&mut self, rule: RuleRef) {
        if rule.rhs.is_empty() {
            while self.nulling.len() <= rule.lhs.into() {
                // We don't know whether `H` is `Clone`.
                self.nulling.push(None);
            }
            // Add a rule of the form `LHS ⸬= ε`.
            assert!(
                self.nulling[rule.lhs.usize()].is_none(),
                "Duplicate nulling rule"
            );
            self.nulling[rule.lhs.usize()] = Some(
                self.add_history_node(
                    HistoryNodeBinarize {
                        prev: rule.history_id,
                        depth: 0,
                    }
                    .into(),
                ),
            );
        } else {
            // Rewrite to a set of binarized rules.
            // From `LHS ⸬= A B C … X Y Z` to:
            // ____________________
            // | LHS ⸬= S0  Z
            // | S0  ⸬= S1  Y
            // | S1  ⸬= S2  X
            // | …
            // | Sm  ⸬= Sn  C
            // | Sn  ⸬= A   B
            let mut rhs_iter = rule.rhs.iter().cloned();
            let sym_range = cmp::max(rule.rhs.len(), 2) - 2;
            let left_iter = self
                .sym_source
                .generate()
                .take(sym_range)
                .chain(rhs_iter.next());
            let right_iter = rhs_iter.rev().map(Some).chain(iter::once(None));

            let mut next_lhs = rule.lhs;
            let history_graph = &mut self.history_graph;
            let make_rule = |(depth, (left, right)): (usize, _)| {
                let lhs = next_lhs;
                next_lhs = left;
                BinarizedRule {
                    lhs: lhs,
                    rhs: if let Some(right) = right {
                        Two([left, right])
                    } else {
                        One([left])
                    },
                    history_id: history_graph.add_history_node(
                        HistoryNodeBinarize {
                            prev: rule.history_id,
                            depth: depth as u32,
                        }
                        .into(),
                    ),
                }
            };
            self.rules
                .extend(left_iter.zip(right_iter).enumerate().map(make_rule));
        }
    }

    fn rules<'a>(&'a self) -> impl Iterator<Item = RuleRef<'a>>
    where
        Self: 'a,
    {
        self.nulling
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, maybe_history)| {
                maybe_history.map(|history_id| RuleRef {
                    lhs: i.into(),
                    rhs: &[],
                    history_id,
                })
            })
            .chain(self.rules.iter().map(|rule| rule.as_rule_ref()))
    }
}

impl AsRuleRef for BinarizedRule {
    fn as_rule_ref(&self) -> RuleRef {
        RuleRef {
            lhs: self.lhs,
            rhs: match self.rhs {
                One(ref slice) => slice,
                Two(ref slice) => slice,
            },
            history_id: self.history_id,
        }
    }
}

impl BinarizedRule {
    /// Creates a new binarized rule.
    pub fn new(rule: RuleRef) -> Self {
        BinarizedRule {
            history_id: rule.history_id,
            lhs: rule.lhs,
            rhs: if rule.rhs.len() == 1 {
                One([rule.rhs[0]])
            } else if rule.rhs.len() == 2 {
                Two([rule.rhs[0], rule.rhs[1]])
            } else {
                panic!("invalid rule rhs length")
            },
        }
    }

    /// Returns the first symbol.
    pub fn lhs(&self) -> Symbol {
        self.lhs
    }

    /// Returns the first symbol.
    pub fn rhs0(&self) -> Symbol {
        match self.rhs {
            One(slice) => slice[0],
            Two(slice) => slice[0],
        }
    }

    /// Returns the second symbol, if present.
    pub fn rhs1(&self) -> Option<Symbol> {
        match self.rhs {
            One(_) => None,
            Two(slice) => Some(slice[1]),
        }
    }
}

impl PartialEq for BinarizedRule {
    fn eq(&self, other: &Self) -> bool {
        (self.lhs, &self.rhs) == (other.lhs, &other.rhs)
    }
}

impl PartialOrd for BinarizedRule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.lhs, &self.rhs).partial_cmp(&(other.lhs, &other.rhs))
    }
}

impl Ord for BinarizedRule {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.lhs, &self.rhs).cmp(&(other.lhs, &other.rhs))
    }
}

impl Eq for BinarizedRule {}
