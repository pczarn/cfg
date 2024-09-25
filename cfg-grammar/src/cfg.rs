use std::cell::RefCell;
use std::cmp;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::{mem, ops};

use occurence_map::OccurenceMap;
use rule_builder::RuleBuilder;

use crate::local_prelude::*;
use cfg_history::{
    BinarizedRhsRange::*, HistoryGraph, HistoryId, HistoryNode, HistoryNodeBinarize,
    HistoryNodeEliminateNulling, LinkedHistoryNode, RootHistoryNode,
};

/// Representation of context-free grammars.
#[derive(Clone, Debug)]
pub struct Cfg {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<CfgRule>,
    /// Start symbols.
    roots: MaybeSmallVec<Symbol>,
    wrapped_roots: MaybeSmallVec<WrappedRoot, 2>,
    /// History container.
    history_graph: HistoryGraph,
    rhs_len_invariant: Option<usize>,
    eliminate_nulling: bool,
    tmp_stack: RefCell<Vec<Symbol>>,
}

/// Your standard grammar rule representation.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct CfgRule {
    /// The rule's left-hand side symbol.
    pub lhs: Symbol,
    /// The rule's right-hand side symbols.
    pub rhs: Rc<[Symbol]>,
    /// The rule's history.
    pub history_id: HistoryId,
}

#[derive(Clone, Copy, Debug)]
pub struct WrappedRoot {
    pub start_of_input: Symbol,
    pub root: Symbol,
    pub end_of_input: Symbol,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum RhsPropertyMode {
    All,
    Any,
}

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
            roots: MaybeSmallVec::new(),
            wrapped_roots: MaybeSmallVec::new(),
            history_graph,
            rhs_len_invariant: None,
            eliminate_nulling: false,
            tmp_stack: RefCell::new(vec![]),
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

    pub fn set_roots(&mut self, roots: impl AsRef<[Symbol]>) {
        self.roots = roots.as_ref().iter().copied().collect();
    }

    pub fn roots(&self) -> &[Symbol] {
        &self.roots[..]
    }

    pub fn wrapped_roots(&self) -> &[WrappedRoot] {
        &self.wrapped_roots[..]
    }

    pub fn has_roots(&self) -> bool {
        !self.roots.is_empty()
    }

    /// Modifies this grammar to its weak equivalent.
    ///
    /// # Invariants
    ///
    /// All rule RHS' have at most `n` symbols at all times until another
    /// call to this method or a call to [`fn binarize_and_eliminate_nulling_rules`].
    ///
    /// [`fn binarize_and_eliminate_nulling_rules`]: Self::binarize_and_eliminate_nulling_rules
    pub fn limit_rhs_len(&mut self, limit: Option<usize>) {
        self.rhs_len_invariant = limit;
        let mut container = mem::take(&mut self.rules);
        container.retain(|rule| self.maybe_process_rule(rule));
        self.rules.extend(container);
    }

    pub fn rule_rhs_len_allowed_range(&self) -> ops::Range<usize> {
        self.eliminate_nulling as usize..self.rhs_len_invariant.unwrap_or(usize::MAX)
    }

    /// Sorts the rule array.
    pub fn sort(&mut self) {
        self.rules.sort();
    }

    /// Sorts the rule array in place, using the argument to compare elements.
    pub fn sort_by<F>(&mut self, compare: impl FnMut(&CfgRule, &CfgRule) -> cmp::Ordering) {
        self.rules.sort_by(compare);
    }

    /// Removes consecutive duplicate rules.
    pub fn dedup(&mut self) {
        self.rules.dedup();
    }

    pub fn extend(&mut self, other: &Cfg) {
        let mut map = BTreeMap::new();
        let mut work_stack: Vec<_> = other.rules().map(|rule| rule.history_id).collect();
        let new_nodes_start = self.history_graph.len();
        while let Some(others_history_id) = work_stack.pop() {
            map.entry(others_history_id).or_insert_with(|| {
                let node = other.history_graph[others_history_id.get()].clone();
                if let HistoryNode::Linked { prev, .. } = node {
                    work_stack.push(prev);
                }
                self.add_history_node(node)
            });
        }
        for node in &mut self.history_graph[new_nodes_start..] {
            match node {
                &mut HistoryNode::Linked { ref mut prev, .. } => {
                    *prev = map.get(prev).copied().expect("history ID not found");
                }
                HistoryNode::Root(..) => {}
            }
        }
        self.rules
            .extend(other.rules.iter().cloned().map(|mut cfg_rule| {
                cfg_rule.history_id = map
                    .get(&cfg_rule.history_id)
                    .copied()
                    .expect("history ID not found");
                cfg_rule
            }));
    }

    /// Ensures the grammar is binarized and eliminates all nulling rules, which have the
    /// form `A ::= epsilon`. Returns the eliminated parts of the grammar as a nulling subgrammar.
    ///
    /// In other words, this method splits off the nulling parts of the grammar.
    ///
    /// The language represented by the grammar is preserved, except for the possible lack of
    /// the empty string. Unproductive rules aren't preserved.
    ///
    /// # Invariants
    ///
    /// All rule RHS' have at least 1 symbol and at most 2 symbols at all times until
    /// another call to this method or a call to [`fn limit_rhs_len`].
    ///
    /// [`fn limit_rhs_len`]: Self::limit_rhs_len
    pub fn binarize_and_eliminate_nulling_rules(&mut self) -> Cfg {
        self.limit_rhs_len(Some(2));

        let mut result = Cfg::with_sym_source_and_history_graph(
            self.sym_source.clone(),
            self.history_graph.clone(),
        );

        let mut nullable = self.nulling_set();
        self.rhs_closure_for_all(&mut nullable);
        if nullable.iter().count() == 0 {
            return result;
        }

        let mut rewritten_work = Cfg::new();
        for rule in self.rules() {
            let is_nullable = |sym: &Symbol| nullable[*sym];
            let maybe_which = match (
                rule.rhs.get(0).map(is_nullable),
                rule.rhs.get(1).map(is_nullable),
            ) {
                (Some(true), Some(true)) | (Some(true), None) => Some(All),
                (Some(true), Some(false)) => Some(Left),
                (Some(false), Some(true)) => Some(Right),
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
                    if rule.rhs.len() == 2 {
                        rewritten_work
                            .rule(rule.lhs)
                            .rhs(&rule.rhs[0..1])
                            .history(history_id);
                        rewritten_work
                            .rule(rule.lhs)
                            .rhs(&rule.rhs[1..2])
                            .history(history_id);
                    }
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
        self.rhs_closure_for_all(&mut productive);
        self.rules.retain(|rule| {
            // Retain the rule only if it's productive. We have to, in order to remove rules
            // that were made unproductive as a result of `A ::= epsilon` rule elimination.
            // Otherwise, some of our nonterminal symbols might be terminal.
            productive[rule.lhs]
        });

        result
    }

    pub fn rules<'a>(&'a self) -> impl Iterator<Item = &CfgRule>
    where
        Self: 'a,
    {
        self.rules.iter()
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

    pub fn retain(&mut self, f: impl FnMut(&CfgRule) -> bool) {
        self.rules.retain(f);
    }

    fn maybe_process_rule(&mut self, rule: &CfgRule) -> bool {
        if self.rule_rhs_len_allowed_range().contains(&rule.rhs.len()) {
            return true;
        }

        // Rewrite to a set of binarized rules.
        // From `LHS ⸬= A B C … X Y Z` to:
        // ____________________
        // | LHS ⸬= S0  Z
        // | S0  ⸬= S1  Y
        // | S1  ⸬= S2  X
        // | …
        // | Sm  ⸬= Sn  C
        // | Sn  ⸬= A   B
        let mut rhs_rev = rule.rhs.to_vec();
        rhs_rev.reverse();
        let mut tail = Vec::new();
        let mut i: u32 = 0;
        while !rhs_rev.is_empty() {
            let tail_idx = rhs_rev.len().saturating_sub(self.rule_rhs_len_allowed_range().end);
            tail.extend(rhs_rev.drain(tail_idx..));
            tail.reverse();
            let lhs;
            if rhs_rev.is_empty() {
                lhs = rule.lhs;
            } else {
                lhs = self.next_sym();
                rhs_rev.push(lhs);
            }
            let history_id;
            if i == 0 && rhs_rev.is_empty() {
                history_id = rule.history_id;
            } else {
                let history_node_binarize = HistoryNodeBinarize {
                    prev: rule.history_id,
                    height: i,
                    is_top: rhs_rev.is_empty(),
                };
                history_id = self.history_graph.add_history_node(history_node_binarize.into());
            }
            self.rules.push(
                CfgRule::new(
                    lhs,
                    &tail[..],
                    history_id
                )
            );
            tail.clear();
            i += 1;
        }

        false
    }

    pub fn add_rule(&mut self, rule: CfgRule) {
        if self.maybe_process_rule(&rule) {
            self.rules.push(rule);
        }
    }

    /// Reverses the grammar.
    pub fn reverse(&mut self) {
        for rule in &mut self.rules {
            rule.rhs = rule.rhs.iter().copied().rev().collect();
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

    pub fn rhs_closure_for_all(&self, property: &mut SymbolBitSet) {
        self.rhs_closure(property, RhsPropertyMode::All)
    }

    pub fn rhs_closure_for_any(&self, property: &mut SymbolBitSet) {
        self.rhs_closure(property, RhsPropertyMode::Any)
    }

    pub fn rhs_closure(&self, property: &mut SymbolBitSet, property_mode: RhsPropertyMode) {
        let mut tmp_stack = self.tmp_stack.borrow_mut();
        tmp_stack.extend(property.iter());

        let occurence_map = OccurenceMap::from_rules(self.rules());

        while let Some(work_sym) = tmp_stack.pop() {
            for &rule_id in occurence_map.get(work_sym).rhs() {
                let rule = &self.rules[rule_id];
                let mut rhs_iter = rule.rhs.iter();
                let get_property = |&sym: &Symbol| property[sym];
                let rhs_satifies_property = match property_mode {
                    RhsPropertyMode::All => rhs_iter.all(get_property),
                    RhsPropertyMode::Any => rhs_iter.any(get_property),
                };
                if !property[rule.lhs] && rhs_satifies_property {
                    property.set(rule.lhs, true);
                    tmp_stack.push(rule.lhs);
                }
            }
        }
    }

    pub fn rhs_closure_with_values(&mut self, value: &mut [Option<u32>]) {
        let mut tmp_stack = self.tmp_stack.borrow_mut();
        for (sym_id, maybe_sym_value) in value.iter().enumerate() {
            if maybe_sym_value.is_some() {
                tmp_stack.push(Symbol::from(sym_id));
            }
        }

        let occurence_map = OccurenceMap::from_rules(self.rules());

        while let Some(work_sym) = tmp_stack.pop() {
            let rules = occurence_map
                .get(work_sym)
                .rhs()
                .iter()
                .map(|&rule_id| &self.rules[rule_id]);
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
                    tmp_stack.push(rule.lhs);
                }
            }
        }
    }

    pub fn wrap_input(&mut self) {
        self.wrapped_roots.clear();
        let roots_len = self.roots.len();
        let roots = mem::replace(&mut self.roots, MaybeSmallVec::with_capacity(roots_len));
        for root in roots {
            let [new_root, start_of_input, end_of_input] = self.sym_source.sym();
            let history_id = self.add_history_node(RootHistoryNode::NoOp.into());
            self.add_rule(CfgRule {
                lhs: new_root,
                rhs: [start_of_input, root, end_of_input].into(),
                history_id,
            });
            self.wrapped_roots.push(WrappedRoot {
                root,
                start_of_input,
                end_of_input,
            });
            self.roots.push(new_root);
        }
    }

    pub fn is_empty(&self) -> bool {
        if self.wrapped_roots.is_empty() {
            self.rules.is_empty()
        } else {
            let mut roots = self.roots.clone();
            roots.sort();
            self.rules()
                .all(|rule| roots.binary_search(&rule.lhs).is_ok())
        }
    }
}

impl CfgRule {
    /// Creates a new rule.
    pub fn new(lhs: Symbol, rhs: impl AsRef<[Symbol]>, history_id: HistoryId) -> Self {
        CfgRule {
            lhs,
            rhs: rhs.as_ref().into(),
            history_id,
        }
    }
}
