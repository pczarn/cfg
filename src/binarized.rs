//! Binarized rules are rules that have at most two symbols on the right-hand side.
//! A binarized grammar contains only such rules.

use std::cmp::{self, Ord, Ordering};
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::slice;

use bit_vec::BitVec;

use crate::analysis::RhsClosure;
use crate::history::{BinarizedRhsSubset::*, HistoryNodeBinarize, HistoryNodeEliminateNulling};
use crate::history::{HistoryGraph, HistoryId, HistoryNode};
use crate::prelude::*;
use crate::rule::{GrammarRule, RuleRef};
use crate::symbol::source::SymbolContainer;

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
    pub fn from_context_free<'a, G>(this: &'a G) -> BinarizedCfg
    where
        G: RuleContainer + Default,
        &'a G: RuleContainerRef<'a, Target = G>,
    {
        let mut new_rule_count = 0;
        // Calculate rule vec capacity.
        for rule in this.rules() {
            if !rule.rhs().is_empty() {
                new_rule_count += 1.max(rule.rhs().len() - 1);
            }
        }
        // Create a new grammar.
        let mut grammar = BinarizedCfg::with_sym_source(this.sym_source().clone());
        grammar.history_graph = this.history_graph().clone();
        grammar.rules = Vec::with_capacity(new_rule_count);
        // Insert all rules from one grammar into the other.
        for rule in this.rules() {
            grammar
                .rule(rule.lhs())
                .rhs_with_history(rule.rhs(), rule.history_id());
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
            for rule in &self.rules {
                let left_nullable = nullable[rule.rhs0().into()];
                let right_nullable = rule.rhs1().map_or(true, |s| nullable[s.into()]);
                if left_nullable && right_nullable {
                    // The rule is copied to the nulling grammar.
                    let prev = rule.history_id();
                    let history = nulling_grammar.add_history_node(
                        HistoryNodeEliminateNulling {
                            prev,
                            rhs0: rule.rhs0(),
                            rhs1: rule.rhs1(),
                            which: All,
                        }
                        .into(),
                    );
                    nulling_grammar
                        .rule(rule.lhs())
                        .rhs_with_history(rule.rhs(), history);
                }
                if rule.rhs().len() == 2 {
                    let history_graph = &mut self.history_graph;
                    let mut make_rule = |sym, which| {
                        let prev = rule.history_id();
                        let history_id = history_graph.add_history_node(
                            HistoryNodeEliminateNulling {
                                prev,
                                rhs0: rule.rhs0(),
                                rhs1: rule.rhs1(),
                                which,
                            }
                            .into(),
                        );
                        rewritten_rules.push(BinarizedRule::new(rule.lhs(), &[sym], history_id));
                    };
                    if left_nullable {
                        make_rule(rule.rhs()[1], Left);
                    }
                    if right_nullable {
                        make_rule(rule.rhs()[0], Right);
                    }
                }
            }
            self.rules.extend(rewritten_rules);
            for rule in self.rules() {
                productive.set(rule.lhs().into(), true);
            }
            for rule in nulling_grammar.rules() {
                productive.set(rule.lhs().into(), false);
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
        F: FnMut(Symbol, &[Symbol], HistoryId) -> bool,
    {
        self.rules
            .retain(|rule| f(rule.lhs(), rule.rhs(), rule.history_id()));
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history_id: HistoryId) {
        if rhs.is_empty() {
            while self.nulling.len() <= lhs.into() {
                // We don't know whether `H` is `Clone`.
                self.nulling.push(None);
            }
            // Add a rule of the form `LHS ⸬= ε`.
            assert!(
                self.nulling[lhs.usize()].is_none(),
                "Duplicate nulling rule"
            );
            self.nulling[lhs.usize()] = Some(
                self.add_history_node(
                    HistoryNodeBinarize {
                        prev: history_id,
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
            let mut rhs_iter = rhs.iter().cloned();
            let sym_range = cmp::max(rhs.len(), 2) - 2;
            let left_iter = self
                .sym_source
                .generate()
                .take(sym_range)
                .chain(rhs_iter.next());
            let right_iter = rhs_iter.rev().map(Some).chain(iter::once(None));

            let mut next_lhs = lhs;
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
                            prev: history_id,
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

/// Iterator over binarized rules.
pub type BinarizedRules<'a> = iter::Chain<
    LhsWithHistoryToRuleRef<'a, iter::Enumerate<iter::Cloned<slice::Iter<'a, Option<HistoryId>>>>>,
    BinarizedRuleToRuleRef<slice::Iter<'a, BinarizedRule>>,
>;

impl<'a> RuleContainerRef<'a> for &'a BinarizedCfg {
    type RuleRef = RuleRef<'a>;
    type Rules = BinarizedRules<'a>;

    fn rules(self) -> Self::Rules {
        LhsWithHistoryToRuleRef::new(self.nulling.iter().cloned().enumerate())
            .chain(BinarizedRuleToRuleRef::new(self.rules.iter()))
    }
}

impl<'a> RuleContainerMut<'a> for &'a mut BinarizedCfg {}

impl GrammarRule for BinarizedRule {
    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        match self.rhs {
            One(ref slice) => slice,
            Two(ref slice) => slice,
        }
    }

    fn history_id(&self) -> HistoryId {
        self.history_id
    }
}

impl BinarizedRule {
    /// Creates a new binarized rule.
    pub fn new(lhs: Symbol, rhs: &[Symbol], history_id: HistoryId) -> Self {
        BinarizedRule {
            history_id,
            lhs: lhs,
            rhs: if rhs.len() == 1 {
                One([rhs[0]])
            } else if rhs.len() == 2 {
                Two([rhs[0], rhs[1]])
            } else {
                unreachable!()
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

/// A wrapper for iteration over rule refs.
pub struct BinarizedRuleToRuleRef<I> {
    iter: I,
}

impl<I> BinarizedRuleToRuleRef<I> {
    /// Creates a new BinarizedRuleToRuleRef.
    pub fn new(iter: I) -> Self {
        BinarizedRuleToRuleRef { iter: iter }
    }
}

impl<'a, I, R> Iterator for BinarizedRuleToRuleRef<I>
where
    I: Iterator<Item = &'a R>,
    R: GrammarRule + 'a,
{
    type Item = RuleRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|rule| rule.as_ref())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// A type for iteration over rule refs.
pub type LhsWithHistory<'a> = (usize, Option<HistoryId>);

/// A wrapper for iteration over rule refs.
pub struct LhsWithHistoryToRuleRef<'a, I>
where
    I: Iterator<Item = LhsWithHistory<'a>>,
{
    iter: I,
    marker: PhantomData<&'a ()>,
}

impl<'a, I> LhsWithHistoryToRuleRef<'a, I>
where
    I: Iterator<Item = LhsWithHistory<'a>>,
{
    /// Creates a new LhsWithHistoryToRuleRef.
    pub fn new(iter: I) -> Self {
        LhsWithHistoryToRuleRef {
            iter,
            marker: PhantomData,
        }
    }
}

impl<'a, I> Iterator for LhsWithHistoryToRuleRef<'a, I>
where
    I: Iterator<Item = LhsWithHistory<'a>>,
{
    type Item = RuleRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for (lhs, history_opt) in &mut self.iter {
            if let Some(history_id) = history_opt {
                return Some(RuleRef {
                    lhs: Symbol::from(lhs),
                    rhs: &[],
                    history_id,
                });
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
