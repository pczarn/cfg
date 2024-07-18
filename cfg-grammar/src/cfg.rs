use std::cmp::{self, Ordering};
use std::collections::BTreeMap;
use std::{iter, ops};

#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

use crate::local_prelude::*;
use crate::rhs_closure::RhsClosure;
use crate::rule_ref::RuleRef;
use crate::symbol_set::SymbolBitSet;
use cfg_history::{
    BinarizedRhsRange::*, HistoryGraph, HistoryId, HistoryNode, HistoryNodeBinarize,
    HistoryNodeEliminateNulling,
};

#[cfg(not(feature = "smallvec"))]
type MaybeSmallVec<T> = Vec<T>;
#[cfg(feature = "smallvec")]
type MaybeSmallVec<T> = SmallVec<[T; 6]>;

/// Basic representation of context-free grammars.
#[derive(Clone)]
pub struct Cfg {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<CfgRule>,
    /// History container.
    history_graph: HistoryGraph,
    /// Limit rule RHS length.
    rhs_len_invariant: Option<usize>,
    eliminate_nulling: bool,
    /// Cache.
    rhs_derivation_cache: BTreeMap<Symbol, MaybeSmallVec<RuleIndex>>,
    /// Cache.
    lhs_derivation_cache: BTreeMap<Symbol, MaybeSmallVec<RuleIndex>>,
    /// Work stack for RHS closure.
    tmp_stack: Vec<Symbol>,
}

/// Typical grammar rule representation.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct CfgRule {
    pub lhs: Symbol,
    /// The rule's right-hand side.
    pub rhs: MaybeSmallVec<Symbol>,
    /// The rule's history.
    pub history_id: HistoryId,
}

type RuleIndex = usize;
type Index = usize;

impl Default for Cfg {
    fn default() -> Self {
        Self::with_sym_source(SymbolSource::new())
    }
}

impl Cfg {
    /// Creates an empty context-free grammar.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty context-free grammar with the given symbol source.
    pub fn with_sym_source(sym_source: SymbolSource) -> Self {
        Cfg::with_sym_source_and_history_graph(sym_source, HistoryGraph::new())
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

    /// Modifies this grammar to its weak equivalent.
    ///
    /// # Invariants
    ///
    /// If `Some(n)` is passed, then all rule RHS are rewritten to have at most
    /// `n` symbols until another call to this method
    /// or [`fn binarize_and_eliminate_nulling_rules`].
    ///
    /// [`fn binarize_and_eliminate_nulling_rules`]: Self::binarize_and_eliminate_nulling_rules
    pub fn allow_rule_length(&mut self, limit: Option<usize>) {
        self.rhs_len_invariant = limit;
        todo!()
    }

    pub fn rule_len_allowed_range(&self) -> ops::Range<usize> {
        self.eliminate_nulling as usize..self.rhs_len_invariant.unwrap_or(usize::MAX)
    }

    /// Sorts the rule array.
    pub fn sort(&mut self) {
        self.rules.sort();
    }

    /// Sorts the rule array in place, using the argument to compare elements.
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(RuleRef, RuleRef) -> Ordering,
    {
        self.rules
            .sort_by(|a, b| compare(a.as_rule_ref(), b.as_rule_ref()));
    }

    /// Removes consecutive duplicate rules.
    pub fn dedup(&mut self) {
        self.rules.dedup();
    }

    pub fn extend(&mut self, other: &Cfg) {
        self.rules.extend(other.rules.iter().cloned());
        todo!()
    }

    /// Ensures the grammar is binarized and eliminates all rules of the form `A ::= epsilon`.
    /// Returns the eliminated parts of the grammar as a nulling subgrammar.
    ///
    /// In other words, this splits off the set of nulling rules.
    ///
    /// The language represented by the grammar is preserved, except for the possible lack of
    /// the empty string. Unproductive rules aren't preserved.
    ///
    ///
    pub fn binarize_and_eliminate_nulling_rules(&mut self) -> Cfg {
        self.limit_rule_length(Some(2));
        let mut result = Cfg::with_sym_source_and_history_graph(
            self.sym_source.clone(),
            self.history_graph.clone(),
        );

        let num_syms = self.sym_source.num_syms();

        let mut nullable = SymbolBitSet::new();
        nullable.nulling_set(&*self);
        self.rhs_closure(&mut nullable);
        if nullable.iter().all(|elem| !elem) {
            return result;
        }

        let mut rewritten_work = Cfg::new();
        for rule in self.rules() {
            let is_nullable = |sym: Symbol| nullable[sym];
            let maybe_which = match (
                is_nullable(rule.rhs[0]),
                rule.rhs.get(1).cloned().map(is_nullable),
            ) {
                (true, Some(true)) | (true, None) => Some(All),
                (true, Some(false)) => Some(Left),
                (false, Some(true)) => Some(Right),
                _ => None,
            };
            if let Some(which) = maybe_which {
                let history = HistoryNodeEliminateNulling {
                    prev: rule.history_id,
                    rhs0: rule.rhs[0],
                    rhs1: rule.rhs.get(1).cloned(),
                    which,
                };
                if which == All {
                    result
                        .rule(rule.lhs)
                        .rhs(rule.rhs[which.as_range()])
                        .history(history);
                } else {
                    rewritten_work
                        .rule(rule.lhs)
                        .rhs(rule.rhs[which.as_range()])
                        .history(history);
                }
            }
        }

        self.extend(&rewritten_work);

        let mut productive = SymbolBitSet::new();
        // TODO check if correct
        productive.productive_set(&*self); // true
        productive.productive_set(&result); // false
        RhsClosure::new(self).rhs_closure(&mut productive);
        self.rules.retain(|rule| {
            // Retain the rule only if it's productive. We have to, in order to remove rules
            // that were made unproductive as a result of `A ::= epsilon` rule elimination.
            // Otherwise, some of our nonterminal symbols might be terminal.

            productive[rule.lhs]
        });

        result

        //     if self.nulling.iter().any(|h| h.is_some()) {
        //         let mut nulling = mem::replace(&mut self.nulling, vec![]);
        //         let nulling_len = nulling.len();
        //         nulling.extend(iter::repeat(None).take(self.sym_source().num_syms() - nulling_len));
        //         let mut nullable: BitVec = nulling.iter().map(|n| n.is_some()).collect();
        //         let mut productive: BitVec = BitVec::from_elem(self.sym_source().num_syms(), true);
        //         // All rules of the form `A ::= ε` go to the nulling grammar.
        //         nulling_grammar.nulling = nulling;
        //         RhsClosure::new(self).rhs_closure(&mut nullable);
        //         // Add new rules.
        //         let mut rewritten_rules = Vec::new();
        //         for rule in self.rules() {
        //             let left_nullable = nullable[rule.rhs[0].into()];
        //             let right_nullable = rule.rhs.get(1).map_or(true, |s| nullable[s.usize()]);
        //             if left_nullable && right_nullable {
        //                 // The rule is copied to the nulling grammar.
        //                 let prev = rule.history_id;
        //                 let history_id = nulling_grammar.add_history_node(
        //                     HistoryNodeEliminateNulling {
        //                         prev,
        //                         rhs0: rule.rhs[0],
        //                         rhs1: rule.rhs.get(1).cloned(),
        //                         which: All,
        //                     }
        //                     .into(),
        //                 );
        //                 nulling_grammar.add_rule(RuleRef { history_id, ..rule });
        //             }
        //             if rule.as_rule_ref().rhs.len() == 2 {
        //                 if left_nullable {
        //                     let history = HistoryNodeEliminateNulling {
        //                         prev: rule.history_id,
        //                         rhs0: rule.rhs[0],
        //                         rhs1: rule.rhs.get(1).cloned(),
        //                         which: Left,
        //                     };
        //                     let rhs = &rule.rhs[1..2];
        //                     rewritten_rules
        //                         .push((BinarizedRule::new(RuleRef { rhs, ..rule }), history));
        //                 }
        //                 if right_nullable {
        //                     let history = HistoryNodeEliminateNulling {
        //                         prev: rule.history_id,
        //                         rhs0: rule.rhs[0],
        //                         rhs1: rule.rhs.get(1).cloned(),
        //                         which: Right,
        //                     };
        //                     let rhs = &rule.rhs[0..1];
        //                     rewritten_rules
        //                         .push((BinarizedRule::new(RuleRef { rhs, ..rule }), history));
        //                 }
        //             }
        //         }
        //         self.rules.extend(
        //             rewritten_rules
        //                 .into_iter()
        //                 .map(|(mut binarized_rule, history)| {
        //                     binarized_rule.history_id =
        //                         self.history_graph.add_history_node(history.into());
        //                     binarized_rule
        //                 }),
        //         );
        //         for rule in self.rules() {
        //             productive.set(rule.lhs.into(), true);
        //         }
        //         for rule in nulling_grammar.rules() {
        //             productive.set(rule.lhs.into(), false);
        //         }
        //         RhsClosure::new(self).rhs_closure(&mut productive);
        //         self.rules.retain(|rule| {
        //             // Retain the rule only if it's productive. We have to, in order to remove rules
        //             // that were made unproductive as a result of `A ::= epsilon` rule elimination.
        //             // Otherwise, some of our nonterminal symbols might be terminal.
        //             let left_productive = productive[rule.rhs0().into()];
        //             let right_productive = rule.rhs1().map_or(true, |s| productive[s.into()]);
        //             left_productive && right_productive
        //         });
        //     }
        //     nulling_grammar
        // }
    }

    pub fn rules<'a>(&'a self) -> impl Iterator<Item = RuleRef<'a>>
    where
        Self: 'a,
    {
        self.rules.iter().map(|rule| rule.into())
    }

    pub fn history_graph(&self) -> &HistoryGraph {
        &self.history_graph
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

    pub fn sym_source(&self) -> &SymbolSource {
        &self.sym_source
    }

    pub fn sym_source_mut(&mut self) -> &mut SymbolSource {
        &mut self.sym_source
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(RuleRef) -> bool,
    {
        self.rules.retain(|rule| f(rule.as_rule_ref()));
    }

    pub fn add_rule(&mut self, rule_ref: RuleRef) {
        self.rules.push(CfgRule {
            lhs: rule_ref.lhs,
            #[cfg(not(feature = "smallvec"))]
            rhs: rule_ref.rhs.to_vec(),
            #[cfg(feature = "smallvec")]
            rhs: SmallVec::from_slice(rule_ref.rhs),
            history_id: rule_ref.history_id,
        });

        // ---

        fn add_rule(&mut self, rule: RuleRef) {
            if rule.rhs.is_empty() {
                self.nulling.extend(
                    iter::repeat(None)
                        .take((rule.lhs.into() + 1).saturating_sub(self.nulling.len())),
                );
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
                    CfgRule {
                        lhs,
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
    }

    /// Reverses the grammar.
    fn reverse(&self) -> Cfg {
        let mut result = Cfg::with_symbol_source_and_history_graph(self.history_graph().clone());

        for rule in self.rules() {
            let mut rhs = rule.rhs.iter().cloned().collect::<Vec<_>>();
            rhs.reverse();
            let rhs = &rhs[..];
            result.add_rule(RuleRef { rhs, ..rule });
        }
        result
    }

    #[inline]
    fn with_symbol_source_and_history_graph(
        sym_source: SymbolSource,
        history_graph: HistoryGraph,
    ) -> Self {
        Cfg {
            sym_source,
            history_graph,
            ..Default::default()
        }
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.history_graph.next_id();
        self.history_graph.push(node);
        result
    }

    fn add_multiple_history_nodes(&mut self, nodes: impl AsRef<[HistoryNode]>) {}

    /// Starts building a new rule.
    fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut Self> {
        RuleBuilder::new(self).rule(lhs)
    }

    /// Starts building a new precedenced rule.
    fn precedenced_rule(&mut self, lhs: Symbol) -> PrecedencedRuleBuilder<&mut Self> {
        PrecedencedRuleBuilder::new(self, lhs)
    }

    // Calculates the RHS transitive closure.
    pub fn rhs_closure(&mut self, property: &mut SymbolBitSet) {
        self.work_stack.clear();
        self.work_stack.extend(property.iter());

        let inverse_derivation = &self.inverse_derivation[..];
        while let Some(work_sym) = self.work_stack.pop() {
            for derivation in find(inverse_derivation, work_sym) {
                if !property[derivation.rule_ref.lhs]
                    && derivation.rule_ref.rhs.iter().any(|sym| property[sym])
                {
                    property[derivation.rule_ref.lhs] = true;
                    self.work_stack.push(derivation.rule_ref.lhs);
                }
            }
        }
    }

    // Calculates the RHS transitive closure.
    pub fn rhs_closure_with_values(&mut self, value: &mut Vec<Option<u32>>) {
        for (sym_id, maybe_sym_value) in value.iter().enumerate() {
            if maybe_sym_value.is_some() {
                self.work_stack.push(Symbol::from(sym_id));
            }
        }

        let inverse_derivation = &self.inverse_derivation[..];
        while let Some(work_sym) = self.work_stack.pop() {
            for derivation in find(inverse_derivation, work_sym) {
                let maybe_work_value = derivation.rule_ref.rhs.iter().fold(Some(0), |acc, elem| {
                    let elem_value = value[elem.usize()];
                    if let (Some(a), Some(b)) = (acc, elem_value) {
                        Some(a + b)
                    } else {
                        None
                    }
                });
                if let Some(work_value) = maybe_work_value {
                    if let Some(current_value) = value[derivation.rule_ref.lhs.usize()] {
                        if current_value <= work_value {
                            continue;
                        }
                    }
                    value[derivation.rule_ref.lhs.usize()] = Some(work_value);
                    self.work_stack.push(derivation.rule_ref.lhs);
                }
            }
        }
    }
}

impl CfgRule {
    /// Creates a new rule.
    pub fn new(lhs: Symbol, rhs: Vec<Symbol>, history_id: HistoryId) -> Self {
        CfgRule {
            lhs,
            #[cfg(not(feature = "smallvec"))]
            rhs,
            #[cfg(feature = "smallvec")]
            rhs: SmallVec::from_vec(rhs),
            history_id,
        }
    }

    // /// Creates a new rule from an RHS slice.
    // pub fn from_rhs_slice(lhs: Symbol, rhs: &[Symbol], history_id: HistoryId) -> Self {
    //     CfgRule {
    //         lhs,
    //         #[cfg(not(feature = "smallvec"))]
    //         rhs: rhs.to_vec(),
    //         #[cfg(feature = "smallvec")]
    //         rhs: SmallVec::from_slice(rhs),
    //         history_id,
    //     }
    // }
}
