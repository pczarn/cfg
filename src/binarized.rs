use std::cmp::{self, Ord, Ordering};
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::slice;

use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef, ContextFreeMut};
use history::{Binarize, EliminateNulling, NullHistory};
use history::BinarizedRhsSubset::*;
use rhs_closure::RhsClosure;
use rule::{GrammarRule, RuleRef};
use rule::container::RuleContainer;
use symbol::{ConsecutiveSymbols, SymbolSource, GrammarSymbol, TerminalSymbolSet};

use self::BinarizedRuleRhs::*;

/// Representation for grammars where right-hand sides of all rules have at most two symbols.
#[derive(Clone)]
pub struct BinarizedCfg<H = NullHistory, Ss = ConsecutiveSymbols> where Ss: SymbolSource {
    /// The symbol source.
    sym_source: Ss,
    /// The array of rules.
    rules: Vec<BinarizedRule<H, Ss::Symbol>>,
    /// The set of history values carried with the grammar's nulling rules, or empty if the grammar
    /// has no nulling rules.
    nulling: Vec<Option<H>>,
}

/// Compact representation of a binarized rule.
pub struct BinarizedRule<H, Ss> where Ss: GrammarSymbol {
    lhs: Ss,
    rhs: BinarizedRuleRhs<Ss>,
    history: H,
}

/// Compact representation of a binarized rule's RHS.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub enum BinarizedRuleRhs<Ss> where Ss: GrammarSymbol {
    One([Ss; 1]),
    Two([Ss; 2]),
}

impl<H> BinarizedCfg<H> {
    /// Creates a BinarizedCfg.
    pub fn new() -> BinarizedCfg<H> {
        BinarizedCfg::with_sym_source(ConsecutiveSymbols::new())
    }
}

impl<H, Ss> BinarizedCfg<H, Ss> where Ss: SymbolSource {
    /// Creates an empty BinarizedCfg with the given symbol source.
    pub fn with_sym_source(sym_source: Ss) -> BinarizedCfg<H, Ss> {
        BinarizedCfg {
            sym_source: sym_source,
            rules: vec![],
            nulling: vec![],
        }
    }

    /// Creates a BinarizedCfg by binarizing a context-free grammar.
    pub fn from_context_free<'a, G>(this: &'a G) -> BinarizedCfg<H, Ss> where
                G: ContextFree<History=H, Source=Ss>,
                &'a G: ContextFreeRef<'a, Target=G>,
                H: Binarize + Clone + 'static,
                Ss: Clone + SymbolSource<Symbol=G::Symbol> {
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

    /// Removes consecutive duplicate rules.
    pub fn dedup(&mut self) {
        self.rules.dedup();
    }
}

impl<H, Ss> BinarizedCfg<H, Ss> where
            H: Binarize + Clone + EliminateNulling,
            Ss: SymbolSource + Clone {
    /// Eliminates all rules of the form `A ::= epsilon`.
    /// 
    /// In other words, this splits off the set of nulling rules.
    ///
    /// The language represented by the grammar is preserved, except for the possible lack of
    /// the empty string. Unproductive rules aren't preserved.
    pub fn eliminate_nulling_rules(&mut self) -> BinarizedCfg<H, Ss> {
        let mut nulling_grammar = BinarizedCfg::with_sym_source(self.sym_source().clone());

        if self.nulling.iter().any(|h| h.is_some()) {
            let mut nulling = mem::replace(&mut self.nulling, vec![]);
            let nulling_len = nulling.len();
            nulling.extend(iter::repeat(None).take(self.sym_source().num_syms() - nulling_len));
            let mut nullable: BitVec = nulling.iter().map(|n| n.is_some()).collect();
            let mut productive: BitVec = BitVec::from_elem(self.sym_source().num_syms(), true);
            // All rules of the form `A ::= ε` go to the nulling grammar.
            nulling_grammar.nulling = nulling;
            for rule in nulling_grammar.rules().chain(self.rules()) {
                productive.set(rule.lhs().usize(), false);
            }
            RhsClosure::new(self).rhs_closure(&mut nullable);
            // Add new rules.
            let mut rewritten_rules = Vec::new();
            for rule in &self.rules {
                let left_nullable = nullable[rule.rhs0().usize()];
                let right_nullable = rule.rhs1().map(|s| nullable[s.usize()]).unwrap_or(true);
                if left_nullable && right_nullable {
                    // The rule is copied to the nulling grammar.
                    let history = rule.history().eliminate_nulling(rule, All);
                    nulling_grammar.rule(rule.lhs()).rhs_with_history(rule.rhs(), history);
                }
                if rule.rhs().len() == 2 {
                    let which = &[(rule.rhs()[0], Right), (rule.rhs()[1], Left)];
                    let with_rhs0 = right_nullable as usize;
                    let with_rhs1 = left_nullable as usize;
                    for &(sym, direction) in &which[1 - with_rhs0 .. 1 + with_rhs1] {
                        rewritten_rules.push(BinarizedRule::new(
                            rule.lhs(),
                            &[sym],
                            rule.history().eliminate_nulling(rule, direction),
                        ));
                    }
                }
            }
            self.rules.extend(rewritten_rules);
            RhsClosure::new(self).rhs_closure(&mut productive);
            self.rules.retain(|rule| {
                // Retain the rule only if it's productive. We have to, in order to remove rules
                // that were made unproductive as a result of `A ::= epsilon` rule elimination.
                // Otherwise, some of our nonterminal symbols might  terminal.
                let left_productive = productive[rule.rhs0().usize()];
                let right_productive = rule.rhs1().map(|s| productive[s.usize()]).unwrap_or(true);
                left_productive && right_productive
            });
        }

        nulling_grammar
    }
}

impl<H, Ss> ContextFree for BinarizedCfg<H, Ss> where
            H: Binarize,
            Ss: SymbolSource {
    type Source = Ss;

    fn sym_source(&self) -> &Ss {
        &self.sym_source
    }

    fn binarize<'a>(&'a self) -> Self where
                &'a Self: ContextFreeRef<'a, Target=Self>,
                H: Clone,
                Ss: Clone {
        // This grammar is already binarized.
        self.clone()
    }
}

impl<'a, H, Ss> ContextFreeRef<'a> for &'a BinarizedCfg<H, Ss> where
            H: Binarize + 'a,
            Ss: SymbolSource,
            Ss::Symbol: 'a {
    type RuleRef = RuleRef<'a, H, Ss::Symbol>;
    type Rules =    iter::Chain<
                        LhsWithHistoryToRuleRef<
                            iter::Enumerate<
                                slice::Iter<'a, Option<H>>
                            >,
                            Ss::Symbol
                        >,
                        BinarizedRuleToRuleRef<
                            slice::Iter<'a, BinarizedRule<H, Ss::Symbol>>
                        >
                    >;

    fn rules(self) -> Self::Rules {
        LhsWithHistoryToRuleRef::new(self.nulling.iter().enumerate())
            .chain(BinarizedRuleToRuleRef::new(self.rules.iter()))
    }
}

impl<'a, H, Ss> ContextFreeMut<'a> for &'a mut BinarizedCfg<H, Ss> where
            H: Binarize + 'a,
            Ss: SymbolSource + 'a,
            Ss::Symbol: 'a {
}

impl<H, Ss> RuleContainer for BinarizedCfg<H, Ss>
        where
            H: Binarize,
            Ss: SymbolSource,
            Ss::Symbol: GrammarSymbol {
    type History = H;

    fn retain<F>(&mut self, mut f: F) where
                F: FnMut(Self::Symbol, &[Self::Symbol], &Self::History) -> bool {
        self.rules.retain(|rule| f(rule.lhs(), rule.rhs(), rule.history()));
    }

    fn add_rule(&mut self, lhs: Self::Symbol, rhs: &[Self::Symbol], history: Self::History) {
        let this_rule_ref = RuleRef {
            lhs: lhs,
            rhs: rhs,
            history: &(),
        };
        self.sym_source.mark_as_nonterminal(lhs);
        if rhs.is_empty() {
            while self.nulling.len() <= lhs.usize() {
                // We don't know whether`H` is `Clone`.
                self.nulling.push(None);
            }
            // Add a rule of the form `LHS ⸬= ε`.
            assert!(self.nulling[lhs.usize()].is_none(), "Duplicate nulling rule");
            self.nulling[lhs.usize()] = Some(history.binarize(&this_rule_ref, 0));
        } else {
            // Rewrite to a set of binarized rules.
            // From `LHS ⸬= A B C … X Y Z` to:
            // ____________________
            //| LHS ⸬= S0  Z
            //| S0  ⸬= S1  Y
            //| S1  ⸬= S2  X
            //| …
            //| Sm  ⸬= Sn  C
            //| Sn  ⸬= A   B
            let mut rhs_iter = rhs.iter().cloned();
            let sym_range = cmp::max(rhs.len(), 2) - 2;
            let left_iter = self.sym_source.nonterminals().take(sym_range).chain(rhs_iter.next());
            let right_iter = rhs_iter.rev().map(Some).chain(iter::once(None));

            let mut next_lhs = lhs;

            self.rules.extend(
                left_iter.zip(right_iter).enumerate().map(
                    |(depth, (left, right))| {
                        let lhs = next_lhs;
                        next_lhs = left;
                        BinarizedRule {
                            lhs: lhs,
                            rhs: if let Some(r) = right { Two([left, r]) } else { One([left]) },
                            history: history.binarize(&this_rule_ref, depth),
                        }
                    }
                )
            );
        }
    }
}

impl<H, Ss> SymbolSource for BinarizedCfg<H, Ss> where Ss: SymbolSource {
    type Symbol = Ss::Symbol;

    fn next_sym(&mut self, terminal: bool) -> Self::Symbol {
        self.sym_source.next_sym(terminal)
    }

    fn mark_as_nonterminal(&mut self, sym: Self::Symbol) {
        self.sym_source.mark_as_nonterminal(sym)
    }

    fn num_syms(&self) -> usize {
        self.sym_source.num_syms()
    }
}

impl<H, Ss> TerminalSymbolSet for BinarizedCfg<H, Ss> where Ss: TerminalSymbolSet {
    fn is_terminal(&self, sym: Self::Symbol) -> bool {
        self.sym_source.is_terminal(sym)
    }
}

impl<H, Ss> GrammarRule for BinarizedRule<H, Ss> where Ss: GrammarSymbol {
    type Symbol = Ss;
    type History = H;

    fn lhs(&self) -> Ss {
        self.lhs
    }

    fn rhs(&self) -> &[Ss] {
        match self.rhs {
            One(ref slice) => slice,
            Two(ref slice) => slice,
        }
    }

    fn history(&self) -> &H {
        &self.history
    }
}

// Can't derive because of the where clause.
impl<H, Ss> Clone for BinarizedRule<H, Ss>
        where H: Clone,
              Ss: GrammarSymbol {
    fn clone(&self) -> Self {
        BinarizedRule {
            lhs: self.lhs,
            rhs: self.rhs,
            history: self.history.clone(),
        }
    }
}

impl<H, Ss> BinarizedRule<H, Ss> where Ss: GrammarSymbol {
    pub fn new(lhs: Ss, rhs: &[Ss], history: H) -> Self {
        BinarizedRule {
            history: history,
            lhs: lhs,
            rhs: if rhs.len() == 1 {
                One([rhs[0]])
            } else if rhs.len() == 2 {
                Two([rhs[0], rhs[1]])
            } else {
                unreachable!()
            }
        }
    }

    pub fn rhs0(&self) -> Ss {
        match self.rhs {
            One(slice) => slice[0],
            Two(slice) => slice[0],
        }
    }

    pub fn rhs1(&self) -> Option<Ss> {
        match self.rhs {
            One(_) => None,
            Two(slice) => Some(slice[1]),
        }
    }
}

impl<Ss> Copy for BinarizedRuleRhs<Ss> where Ss: GrammarSymbol {}

impl<H, Ss> PartialEq for BinarizedRule<H, Ss> where Ss: GrammarSymbol {
    fn eq(&self, other: &Self) -> bool {
        (self.lhs, &self.rhs) == (other.lhs, &other.rhs)
    }
}

impl<H, Ss> PartialOrd for BinarizedRule<H, Ss> where Ss: GrammarSymbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.lhs, &self.rhs).partial_cmp(&(other.lhs, &other.rhs))
    }
}

impl<H, Ss> Ord for BinarizedRule<H, Ss> where Ss: GrammarSymbol {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.lhs, &self.rhs).cmp(&(other.lhs, &other.rhs))
    }
}

impl<H, Ss> Eq for BinarizedRule<H, Ss> where Ss: GrammarSymbol {}

// Can't derive because of the where clause.
impl<Ss> Clone for BinarizedRuleRhs<Ss> where Ss: GrammarSymbol {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct BinarizedRuleToRuleRef<I> {
    iter: I
}

impl<I> BinarizedRuleToRuleRef<I> {
    pub fn new(iter: I) -> Self {
        BinarizedRuleToRuleRef {
            iter: iter
        }
    }
}

impl<'a, I, R, H, S> Iterator for BinarizedRuleToRuleRef<I> where
            I: Iterator<Item=&'a R>,
            R: GrammarRule<History=H, Symbol=S> + 'a,
            H: 'a,
            S: GrammarSymbol + 'a {
    type Item = RuleRef<'a, H, S>;

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

pub type LhsWithHistory<'a, H> = (usize, &'a Option<H>);

pub struct LhsWithHistoryToRuleRef<I, Ss> {
    iter: I,
    marker: PhantomData<Ss>,
}

impl<I, Ss> LhsWithHistoryToRuleRef<I, Ss> {
    pub fn new(iter: I) -> Self {
        LhsWithHistoryToRuleRef {
            iter: iter,
            marker: PhantomData,
        }
    }
}

impl<'a, I, H, S> Iterator for LhsWithHistoryToRuleRef<I, S> where
            I: Iterator<Item=LhsWithHistory<'a, H>>,
            H: 'a,
            S: GrammarSymbol + 'a {
    type Item = RuleRef<'a, H, S>;

    fn next(&mut self) -> Option<Self::Item> {
        for (lhs, history_opt) in &mut self.iter {
            if let Some(history) = history_opt.as_ref() {
                return Some(RuleRef {
                    lhs: S::from(lhs as u64),
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
