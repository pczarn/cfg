//! Analysis of rule usefulness.

use bit_matrix::{FixedBitVec, FixedBitMatrix};
use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef, ContextFreeMut};
use rhs_closure::RhsClosure;
use rule::GrammarRule;
use rule::container::RuleContainer;
use symbol::{SymbolSource, GrammarSymbol};

/// Contains the information about usefulness of the grammar's rules.
/// Useful rules are both reachable and productive.
pub struct Usefulness<G> {
    grammar: G,
    reachability: FixedBitVec,
    productivity: FixedBitVec,
    has_useless_rules: bool,
}

/// An iterator over the grammar's useless rules.
pub struct UselessRules<'a, G, R> where G: 'a {
    rules: R,
    usefulness: &'a Usefulness<&'a mut G>,
}

/// A reference to a useless rule, together with the reason for its uselessness.
#[derive(Clone, Debug)]
pub struct UselessRule<R> {
    /// Reference to a rule.
    pub rule: R,
    /// Indicates whether the rule is unreachable.
    pub unreachable: bool,
    /// Indicates whether the rule is unproductive.
    pub unproductive: bool,
}

/// Returns the set of used symbols.
fn used_syms<'a, G>(grammar: &'a G) -> FixedBitVec where
            G: ContextFree,
            &'a G: ContextFreeRef<'a, Target=G> {
    let num_syms = grammar.sym_source().num_syms();
    let mut used_syms = BitVec::from_elem(num_syms, false);

    for rule in grammar.rules() {
        used_syms.set(rule.lhs().usize(), true);
        for &sym in rule.rhs() {
            used_syms.set(sym.usize(), true);
        }
    }
    FixedBitVec::from_bit_vec(used_syms)
}

/// Returns the set of productive symbols.
fn productive_syms<'a, G>(grammar: &'a G) -> FixedBitVec where
            G: ContextFree,
            &'a G: ContextFreeRef<'a, Target=G> {
    let num_syms = grammar.sym_source().num_syms();
    let mut productive_syms = BitVec::from_elem(num_syms, true);

    for rule in grammar.rules() {
        if !rule.rhs().is_empty() {
            productive_syms.set(rule.lhs().usize(), false);
        }
    }

    RhsClosure::new(grammar).rhs_closure(&mut productive_syms);
    FixedBitVec::from_bit_vec(productive_syms)
}

/// Returns the set of reachable symbols.
fn reachable_syms<'a, G>(grammar: &'a G) -> FixedBitVec where
            G: ContextFree,
            &'a G: ContextFreeRef<'a, Target=G> {
    let num_syms = grammar.sym_source().num_syms();
    let start_sym = grammar.start_sym().usize();
    let mut reachability = FixedBitMatrix::new(num_syms, num_syms);

    for rule in grammar.rules() {
        reachability.set(rule.lhs().usize(), rule.lhs().usize(), true);
        for &sym in rule.rhs() {
            reachability.set(rule.lhs().usize(), sym.usize(), true);
        }
    }

    reachability.transitive_closure();

    let mut unreachable_syms = FixedBitVec::from_elem(num_syms, false);
    for (syms, block) in unreachable_syms.iter_mut().zip(reachability[start_sym].iter()) {
        *syms = *block;
    }
    unreachable_syms
}

impl<'a, G> Usefulness<&'a mut G> where
            G: ContextFree,
            for<'b> &'b G: ContextFreeRef<'b, Target=G>,
            for<'b> &'b mut G: ContextFreeMut<'b, Target=G> {
    /// Analyzes usefulness of the grammar's rules. In particular, it checks for reachable
    /// and productive symbols.
    pub fn new(grammar: &'a mut G) -> Usefulness<&'a mut G> {
        let productivity = productive_syms(grammar);
        let reachability = reachable_syms(grammar);
        let used_syms = used_syms(grammar);
        let all_useful = productivity.storage().iter()
                            .zip(reachability.storage().iter())
                            .zip(used_syms.storage().iter())
                            .all(|((&productive, &reachable), &used)| {
                                reachable == used && productive & used == used
                            });
        Usefulness {
            grammar: grammar,
            productivity: productivity,
            reachability: reachability,
            has_useless_rules: !all_useful,
        }
    }

    /// Checks whether a symbol is productive. Can be used to determine the precise reason
    /// for a rule's unproductiveness.
    pub fn productivity(&self, sym: G::Symbol) -> bool {
        self.productivity[sym.usize()]
    }

    /// Checks whether the grammar has useless rules.
    pub fn has_useless_rules(&self) -> bool {
        self.has_useless_rules
    }
}

// Watch out: Normal type bounds conflict with HRTB.
impl<'a, G> Usefulness<&'a mut G> where
            G: ContextFree,
            &'a G: ContextFreeRef<'a, Target=G>,
            &'a mut G: ContextFreeMut<'a, Target=G> {
    /// Returns an iterator over the grammar's useless rules.
    pub fn useless_rules(&'a self)
            -> UselessRules<'a, G, <&'a G as ContextFreeRef<'a>>::Rules> {
        UselessRules {
            rules: self.grammar.rules(),
            usefulness: self,
        }
    }

    /// Removes useless rules. The language represented by the grammar doesn't change.
    pub fn remove_useless_rules(&mut self) {
        if self.has_useless_rules {
            let productivity = &self.productivity;
            let reachability = &self.reachability;
            self.grammar.retain(|lhs, rhs, _| {
                let productive = rhs.iter().all(|sym| productivity[sym.usize()]);
                let reachable = reachability[lhs.usize()];
                productive && reachable
            });
        }
    }
}

impl<'a, G> Iterator for UselessRules<'a, G, <&'a G as ContextFreeRef<'a>>::Rules> where
            G: ContextFree + 'a,
            &'a G: ContextFreeRef<'a, Target=G> {
    type Item = UselessRule<<<&'a G as ContextFreeRef<'a>>::Rules as Iterator>::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.usefulness.has_useless_rules {
            return None;
        }

        while let Some(rule) = self.rules.next() {
            let lhs = rule.lhs().usize();
            let productive = rule.rhs().iter().all(|sym| self.usefulness.productivity[sym.usize()]);
            let reachable = self.usefulness.reachability[lhs];

            if !reachable || !productive {
                return Some(UselessRule {
                    rule: rule,
                    unreachable: !reachable,
                    unproductive: !productive,
                });
            }
        }

        None
    }
}
