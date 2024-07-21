use std::cmp::{self, Ordering};
use std::collections::BTreeMap;
use std::{iter, ops};

use rule_builder::RuleBuilder;
#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

use crate::local_prelude::*;
use cfg_history::{
    BinarizedRhsRange::*, HistoryGraph, HistoryId, HistoryNode, HistoryNodeBinarize,
    HistoryNodeEliminateNulling, LinkedHistoryNode, RootHistoryNode,
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
    /// Start symbols.
    roots: MaybeSmallVec<Symbol>,
    /// History container.
    history_graph: HistoryGraph,
    rhs_len_invariant: Option<usize>,
    eliminate_nulling: bool,
    occurence_cache: BTreeMap<Symbol, Occurences>,
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

/// Two (maybe Small) `Vec`s of rule indices.
#[derive(Clone)]
pub struct Occurences {
    lhs: MaybeSmallVec<RuleIndex>,
    rhs: MaybeSmallVec<RuleIndex>,
}

type RuleIndex = usize;

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

    pub fn with_sym_source_and_history_graph(
        sym_source: SymbolSource,
        history_graph: HistoryGraph,
    ) -> Self {
        Cfg {
            sym_source,
            rules: vec![],
            history_graph,
            rhs_len_invariant: None,
            eliminate_nulling: false,
            occurence_cache: BTreeMap::new(),
            tmp_stack: vec![],
        }
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

    pub fn set_start(&mut self, start: Symbol) {
        self.roots = MaybeSmallVec::from_slice(&[symbol]);
    }

    pub fn set_roots(&mut self, roots: &[Symbol]) {
        self.roots
    }

    /// Modifies this grammar to its weak equivalent.
    ///
    /// # Invariants
    ///
    /// `n` symbols until another call to this method
    /// or [`fn binarize_and_eliminate_nulling_rules`].
    ///
    /// [`fn binarize_and_eliminate_nulling_rules`]: Self::binarize_and_eliminate_nulling_rules
    pub fn allow_rule_rhs_len(&mut self, limit: Option<usize>) {
        self.rhs_len_invariant = limit;
        todo!()
    }

    pub fn rule_rhs_len_allowed_range(&self) -> ops::Range<usize> {
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
        self.rules.sort_by(|a, b| compare(a.into(), b.into()));
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
        self.allow_rule_rhs_len(Some(2));

        let mut result = Cfg::with_sym_source_and_history_graph(
            self.sym_source.clone(),
            self.history_graph.clone(),
        );

        let mut nullable = SymbolBitSet::new();
        nullable.nulling(&*self);
        self.rhs_closure(&mut nullable);
        if nullable.iter().count() == 0 {
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
                let history_id = result.add_history_node(
                    HistoryNodeEliminateNulling {
                        prev: rule.history_id,
                        rhs0: rule.rhs[0],
                        rhs1: rule.rhs.get(1).cloned(),
                        which,
                    }
                    .into(),
                );
                if which == All {
                    result
                        .rule(rule.lhs)
                        .rhs(&rule.rhs[which.as_range()])
                        .history(history_id);
                } else {
                    rewritten_work
                        .rule(rule.lhs)
                        .rhs(&rule.rhs[which.as_range()])
                        .history(history_id);
                }
            }
        }

        self.extend(&rewritten_work);

        let mut productive = SymbolBitSet::new();
        // TODO check if correct
        productive.productive(&*self); // true
        productive.productive(&result); // false
        self.rhs_closure(&mut productive);
        self.rules.retain(|rule| {
            // Retain the rule only if it's productive. We have to, in order to remove rules
            // that were made unproductive as a result of `A ::= epsilon` rule elimination.
            // Otherwise, some of our nonterminal symbols might be terminal.
            productive[rule.lhs]
        });

        result
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
        self.rules.retain(|rule| f(rule.into()));
    }

    pub fn add_rule<R: Into<CfgRule>>(&mut self, rule: R) {
        let rule = rule.into();
        if self.rule_rhs_len_allowed_range().contains(&rule.rhs.len()) {
            self.rules.push(rule);
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
                CfgRule::new(
                    lhs,
                    if let Some(right) = right {
                        vec![left, right]
                    } else {
                        vec![left]
                    },
                    history_graph.add_history_node(
                        HistoryNodeBinarize {
                            prev: rule.history_id,
                            depth: depth as u32,
                        }
                        .into(),
                    ),
                )
            };
            self.rules
                .extend(left_iter.zip(right_iter).enumerate().map(make_rule));
        }
    }

    /// Reverses the grammar.
    pub fn reverse(&mut self) {
        for rule in &mut self.rules {
            rule.rhs.reverse();
        }
    }

    #[inline]
    pub fn with_symbol_source_and_history_graph(
        sym_source: SymbolSource,
        history_graph: HistoryGraph,
    ) -> Self {
        Cfg {
            sym_source,
            history_graph,
            ..Default::default()
        }
    }

    pub fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.history_graph.next_id();
        self.history_graph.push(node);
        result
    }

    pub fn add_multiple_history_nodes<const N: usize>(
        &mut self,
        root: RootHistoryNode,
        nodes: [LinkedHistoryNode; N],
    ) -> HistoryId {
        let mut prev = self.add_history_node(HistoryNode::Root(root));
        for node in nodes {
            prev = self.add_history_node(HistoryNode::Linked { prev, node });
        }
        prev
    }

    /// Starts building a new rule.
    pub fn rule(&mut self, lhs: Symbol) -> RuleBuilder {
        RuleBuilder::new(self).rule(lhs)
    }

    /// Starts building a new precedenced rule.
    pub fn precedenced_rule(&mut self, lhs: Symbol) -> PrecedencedRuleBuilder {
        PrecedencedRuleBuilder::new(self, lhs)
    }

    pub fn rhs_closure(&mut self, property: &mut SymbolBitSet) {
        self.tmp_stack.clear();
        self.tmp_stack.extend(property.iter());

        while let Some(work_sym) = self.tmp_stack.pop() {
            if let Some(occurences) = self.occurence_cache.get(&work_sym) {
                for &rule_id in &occurences.rhs {
                    let rule = &self.rules[rule_id];
                    if !property[rule.lhs] && rule.rhs.iter().any(|sym| property[*sym]) {
                        property.set(rule.lhs, true);
                        self.tmp_stack.push(rule.lhs);
                    }
                }
            }
        }
    }

    pub fn rhs_closure_with_values(&mut self, value: &mut [Option<u32>]) {
        for (sym_id, maybe_sym_value) in value.iter().enumerate() {
            if maybe_sym_value.is_some() {
                self.tmp_stack.push(Symbol::from(sym_id));
            }
        }

        // while let Some(work_sym) = self.tmp_stack.pop() {
        //     for rule_id in self
        //         .occurence_cache
        //         .get(&work_sym)
        //         .unwrap_or(&Occurences::new())
        //         .rhs
        //     {
        //         let rule = &self.rules[rule_id];
        //         if !property[rule.lhs] && rule.rhs.iter().any(|sym| property[sym]) {
        //             property.set(rule.lhs, true);
        //             self.tmp_stack.push(rule.lhs);
        //         }
        //     }
        // }
        while let Some(work_sym) = self.tmp_stack.pop() {
            let empty_occurences = Occurences::new();
            let occurences = self
                .occurence_cache
                .get(&work_sym)
                .unwrap_or(&empty_occurences);
            let rules = occurences.rhs.iter().map(|&rule_id| &self.rules[rule_id]);
            for rule in rules {
                let maybe_work_value = rule
                    .rhs
                    .iter()
                    .try_fold(0, |acc, elem| value[elem.usize()].map(|val| acc + val));
                if let Some(work_value) = maybe_work_value {
                    if let Some(current_value) = value[rule.lhs.usize()] {
                        if current_value <= work_value {
                            continue;
                        }
                    }
                    value[rule.lhs.usize()] = Some(work_value);
                    self.tmp_stack.push(rule.lhs);
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

impl Occurences {
    fn new() -> Self {
        Occurences {
            lhs: MaybeSmallVec::new(),
            rhs: MaybeSmallVec::new(),
        }
    }

    pub fn lhs(&self) -> &[RuleIndex] {
        &self.lhs[..]
    }

    pub fn rhs(&self) -> &[RuleIndex] {
        &self.rhs[..]
    }
}
