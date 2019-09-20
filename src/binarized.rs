//! Binarized rules are rules that have at most two symbols on the right-hand side.
//! A binarized grammar contains only such rules.

use std::cmp::{self, Ord, Ordering};
use std::iter;
use std::mem;
use std::slice;

use bit_vec::BitVec;

use analysis::RhsClosure;
use grammar::{ContextFree, ContextFreeRef, ContextFreeMut};
use history::{Binarize, EliminateNulling, NullHistory};
use history::BinarizedRhsSubset::*;
use rule::{GrammarRule, RuleRef};
use rule::container::{RuleContainer, EmptyRuleContainer};
use symbol::{Symbol, SymbolSource};
use symbol::source::SymbolContainer;

use self::BinarizedRuleRhs::*;

/// Representation for grammars where right-hand sides of all rules have at most two symbols.
#[derive(Clone)]
pub struct BinarizedCfg<H = NullHistory> {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<BinarizedRule<H>>,
    /// The set of history values carried with the grammar's nulling rules, or empty if the grammar
    /// has no nulling rules.
    nulling: Vec<Option<H>>,
}

/// Compact representation of a binarized rule.
#[derive(Copy, Clone)]
pub struct BinarizedRule<H> {
    lhs: Symbol,
    rhs: BinarizedRuleRhs,
    history: H,
}

/// Compact representation of a binarized rule's RHS.
#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum BinarizedRuleRhs {
    /// RHS with one symbol.
    One([Symbol; 1]),
    /// RHS with two symbols.
    Two([Symbol; 2]),
}

impl<H> Default for BinarizedCfg<H> {
    fn default() -> Self {
        Self::with_sym_source(SymbolSource::new())
    }
}

impl<H> BinarizedCfg<H> {
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
        }
    }

    /// Creates a BinarizedCfg by binarizing a context-free grammar.
    pub fn from_context_free<'a, G>(this: &'a G) -> BinarizedCfg<H>
        where G: ContextFree<History = H>,
              &'a G: ContextFreeRef<'a, Target = G>,
              H: Binarize + Clone + 'static
    {
        let mut new_rule_count = 0;
        // Calculate rule vec capacity.
        for rule in this.rules() {
            if !rule.rhs().is_empty() {
                new_rule_count += cmp::max(1, rule.rhs().len() - 1);
            }
        }
        // Create a new grammar.
        let mut grammar = BinarizedCfg::with_sym_source(this.sym_source().clone());
        grammar.rules = Vec::with_capacity(new_rule_count);
        // Insert all rules from one grammar into the other.
        for rule in this.rules() {
            grammar.rule(rule.lhs()).rhs_with_history(rule.rhs(), rule.history().clone());
        }

        grammar
    }

    /// Sorts the rule array.
    pub fn sort(&mut self) {
        self.rules.sort();
    }

    /// Sorts the rule array in place, using the argument to compare elements.
    pub fn sort_by<F>(&mut self, compare: F)
        where F: FnMut(&BinarizedRule<H>, &BinarizedRule<H>) -> Ordering
    {
        self.rules.sort_by(compare);
    }

    /// Removes consecutive duplicate rules.
    pub fn dedup(&mut self) {
        self.rules.dedup();
    }
}

impl<H> BinarizedCfg<H>
    where H: Binarize
{
    /// Returns generated symbols.
    pub fn sym<T>(&mut self) -> T
        where T: SymbolContainer
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
}

impl<H> BinarizedCfg<H>
    where H: Binarize + Clone + EliminateNulling
{
    /// Eliminates all rules of the form `A ::= epsilon`.
    ///
    /// In other words, this splits off the set of nulling rules.
    ///
    /// The language represented by the grammar is preserved, except for the possible lack of
    /// the empty string. Unproductive rules aren't preserved.
    pub fn eliminate_nulling_rules(&mut self) -> BinarizedCfg<H> {
        let mut nulling_grammar = BinarizedCfg::with_sym_source(self.sym_source.clone());

        if self.nulling.iter().any(|h| h.is_some()) {
            let mut nulling = mem::replace(&mut self.nulling, vec![]);
            let nulling_len = nulling.len();
            nulling.extend(iter::repeat(None).take(self.sym_source().num_syms() - nulling_len));
            let mut nullable: BitVec = nulling.iter().map(|n| n.is_some()).collect();
            let mut productive: BitVec = BitVec::from_elem(self.sym_source().num_syms(), true);
            // All rules of the form `A ::= ε` go to the nulling grammar.
            nulling_grammar.nulling = nulling;
            for rule in nulling_grammar.rules().chain(self.rules()) {
                productive.set(rule.lhs().into(), false);
            }
            RhsClosure::new(self).rhs_closure(&mut nullable);
            // Add new rules.
            let mut rewritten_rules = Vec::new();
            for rule in &self.rules {
                let left_nullable = nullable[rule.rhs0().into()];
                let right_nullable = rule.rhs1().map_or(true, |s| nullable[s.into()]);
                if left_nullable && right_nullable {
                    // The rule is copied to the nulling grammar.
                    let history = rule.history().eliminate_nulling(rule, All);
                    nulling_grammar.rule(rule.lhs()).rhs_with_history(rule.rhs(), history);
                }
                if rule.rhs().len() == 2 {
                    let which = &[(rule.rhs()[0], Right), (rule.rhs()[1], Left)];
                    let with_rhs0 = right_nullable as usize;
                    let with_rhs1 = left_nullable as usize;
                    for &(sym, direction) in &which[1 - with_rhs0..1 + with_rhs1] {
                        rewritten_rules.push(BinarizedRule::new(rule.lhs(),
                                                                &[sym],
                                                                rule.history()
                                                                    .eliminate_nulling(rule,
                                                                                       direction)));
                    }
                }
            }
            self.rules.extend(rewritten_rules);
            RhsClosure::new(self).rhs_closure(&mut productive);
            self.rules.retain(|rule| {
                // Retain the rule only if it's productive. We have to, in order to remove rules
                // that were made unproductive as a result of `A ::= epsilon` rule elimination.
                // Otherwise, some of our nonterminal symbols might  terminal.
                let left_productive = productive[rule.rhs0().into()];
                let right_productive = rule.rhs1().map_or(true, |s| productive[s.into()]);
                left_productive && right_productive
            });
        }

        nulling_grammar
    }
}

impl<H> ContextFree for BinarizedCfg<H>
    where H: Binarize
{
}

/// Iterator over binarized rules.
pub type BinarizedRules<'a, H> =
    iter::Chain<
        LhsWithHistoryToRuleRef<
            iter::Enumerate<
                slice::Iter<'a, Option<H>>
            >
        >,
        BinarizedRuleToRuleRef<
            slice::Iter<'a, BinarizedRule<H>>
        >
    >;

impl<'a, H> ContextFreeRef<'a> for &'a BinarizedCfg<H>
    where H: Binarize + 'a
{
    type RuleRef = RuleRef<'a, H>;
    type Rules = BinarizedRules<'a, H>;

    fn rules(self) -> Self::Rules {
        LhsWithHistoryToRuleRef::new(self.nulling.iter().enumerate())
            .chain(BinarizedRuleToRuleRef::new(self.rules.iter()))
    }
}

impl<'a, H> ContextFreeMut<'a> for &'a mut BinarizedCfg<H> where H: Binarize + 'a {}

impl<H> RuleContainer for BinarizedCfg<H>
    where H: Binarize
{
    type History = H;

    fn sym_source(&self) -> &SymbolSource {
        &self.sym_source
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        &mut self.sym_source
    }

    fn retain<F>(&mut self, mut f: F)
        where F: FnMut(Symbol, &[Symbol], &Self::History) -> bool
    {
        self.rules.retain(|rule| f(rule.lhs(), rule.rhs(), rule.history()));
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: Self::History) {
        let this_rule_ref = RuleRef {
            lhs: lhs,
            rhs: rhs,
            history: &(),
        };
        if rhs.is_empty() {
            while self.nulling.len() <= lhs.into() {
                // We don't know whether `H` is `Clone`.
                self.nulling.push(None);
            }
            // Add a rule of the form `LHS ⸬= ε`.
            assert!(self.nulling[lhs.usize()].is_none(),
                    "Duplicate nulling rule");
            self.nulling[lhs.usize()] = Some(history.binarize(&this_rule_ref, 0));
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
            let left_iter = self.sym_source.generate().take(sym_range).chain(rhs_iter.next());
            let right_iter = rhs_iter.rev().map(Some).chain(iter::once(None));

            let mut next_lhs = lhs;

            self.rules.extend(left_iter.zip(right_iter)
                                       .enumerate()
                                       .map(|(depth, (left, right))| {
                                           let lhs = next_lhs;
                                           next_lhs = left;
                                           BinarizedRule {
                                               lhs: lhs,
                                               rhs: if let Some(r) = right {
                                                   Two([left, r])
                                               } else {
                                                   One([left])
                                               },
                                               history: history.binarize(&this_rule_ref, depth),
                                           }
                                       }));
        }
    }
}

impl<H> EmptyRuleContainer for BinarizedCfg<H> {
    fn empty(&self) -> Self {
        BinarizedCfg::default()
    }
}

impl<H> GrammarRule for BinarizedRule<H> {
    type History = H;

    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        match self.rhs {
            One(ref slice) => slice,
            Two(ref slice) => slice,
        }
    }

    fn history(&self) -> &H {
        &self.history
    }
}

impl<H> BinarizedRule<H> {
    /// Creates a new binarized rule.
    pub fn new(lhs: Symbol, rhs: &[Symbol], history: H) -> Self {
        BinarizedRule {
            history: history,
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

impl<H> PartialEq for BinarizedRule<H> {
    fn eq(&self, other: &Self) -> bool {
        (self.lhs, &self.rhs) == (other.lhs, &other.rhs)
    }
}

impl<H> PartialOrd for BinarizedRule<H> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.lhs, &self.rhs).partial_cmp(&(other.lhs, &other.rhs))
    }
}

impl<H> Ord for BinarizedRule<H> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.lhs, &self.rhs).cmp(&(other.lhs, &other.rhs))
    }
}

impl<H> Eq for BinarizedRule<H> {}

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

impl<'a, I, R, H> Iterator for BinarizedRuleToRuleRef<I>
    where I: Iterator<Item = &'a R>,
          R: GrammarRule<History = H> + 'a,
          H: 'a
{
    type Item = RuleRef<'a, H>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|rule| {
            RuleRef {
                lhs: rule.lhs(),
                rhs: rule.rhs(),
                history: rule.history(),
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// A type for iteration over rule refs.
pub type LhsWithHistory<'a, H> = (usize, &'a Option<H>);

/// A wrapper for iteration over rule refs.
pub struct LhsWithHistoryToRuleRef<I> {
    iter: I,
}

impl<I> LhsWithHistoryToRuleRef<I> {
    /// Creates a new LhsWithHistoryToRuleRef.
    pub fn new(iter: I) -> Self {
        LhsWithHistoryToRuleRef { iter: iter }
    }
}

impl<'a, I, H> Iterator for LhsWithHistoryToRuleRef<I>
    where I: Iterator<Item = LhsWithHistory<'a, H>>,
          H: 'a
{
    type Item = RuleRef<'a, H>;

    fn next(&mut self) -> Option<Self::Item> {
        for (lhs, history_opt) in &mut self.iter {
            if let Some(history) = history_opt.as_ref() {
                return Some(RuleRef {
                    lhs: Symbol::from(lhs),
                    rhs: &[],
                    history: history,
                });
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
