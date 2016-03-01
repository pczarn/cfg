//! Analysis of rule usefulness.

use bit_matrix::BitMatrix;
use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef, ContextFreeMut};
use rhs_closure::RhsClosure;
use rule::GrammarRule;
use rule::container::RuleContainer;
use rule::terminal_set::TerminalSet;
use symbol::{SymbolSource, GrammarSymbol};

/// Contains the information about usefulness of the grammar's rules.
/// Useful rules are both reachable and productive.
pub struct Usefulness<G> {
    grammar: G,
    reachability: BitMatrix,
    reachable_syms: BitVec,
    productivity: BitVec,
    all_useful: bool,
    all_productive: bool,
}

/// An iterator over the grammar's useless rules.
pub struct UselessRules<'a, G, R>
    where G: 'a
{
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
fn used_syms<'a, G>(grammar: &'a G) -> BitVec
    where G: ContextFree,
          &'a G: ContextFreeRef<'a, Target = G>
{
    let num_syms = grammar.sym_source().num_syms();
    let mut used_syms = BitVec::from_elem(num_syms, false);

    for rule in grammar.rules() {
        used_syms.set(rule.lhs().usize(), true);
        for &sym in rule.rhs() {
            used_syms.set(sym.usize(), true);
        }
    }
    used_syms
}

/// Returns the set of productive symbols.
fn productive_syms<'a, G>(grammar: &'a G) -> BitVec
    where G: ContextFree,
          &'a G: ContextFreeRef<'a, Target = G>
{
    let mut productive_syms = grammar.terminal_set().into_bit_vec();
    RhsClosure::new(grammar).rhs_closure(&mut productive_syms);
    productive_syms
}

/// Returns the reachability matrix.
fn reachability<'a, G>(grammar: &'a G) -> BitMatrix
    where G: ContextFree,
          &'a G: ContextFreeRef<'a, Target = G>
{
    let num_syms = grammar.sym_source().num_syms();
    let mut reachability = BitMatrix::new(num_syms, num_syms);

    for rule in grammar.rules() {
        reachability.set(rule.lhs().usize(), rule.lhs().usize(), true);
        for &sym in rule.rhs() {
            reachability.set(rule.lhs().usize(), sym.usize(), true);
        }
    }

    reachability.transitive_closure();

    reachability
}

impl<'a, G> Usefulness<&'a mut G>
    where G: ContextFree,
          for<'b> &'b G: ContextFreeRef<'b, Target = G>,
          for<'b> &'b mut G: ContextFreeMut<'b, Target = G>
{
    /// Analyzes usefulness of the grammar's rules. In particular, it checks for reachable
    /// and productive symbols.
    pub fn new(grammar: &'a mut G) -> Usefulness<&'a mut G> {
        let mut productivity = productive_syms(grammar);
        let reachability = reachability(grammar);
        let used_syms = used_syms(grammar);
        let mut reachable_syms = BitVec::from_elem(grammar.sym_source().num_syms(), false);

        unsafe {
            for ((productive, reachable), &used) in productivity.storage_mut()
                                                                .iter_mut()
                                                                .zip(reachable_syms.storage_mut()
                                                                                   .iter_mut())
                                                                .zip(used_syms.storage().iter()) {
                *productive |= !used;
                *reachable |= !used;
            }
        }

        let all_productive = productivity.storage().iter().all(|&productive| productive == !0);

        Usefulness {
            grammar: grammar,
            productivity: productivity,
            reachability: reachability,
            reachable_syms: reachable_syms,
            all_useful: false,
            all_productive: all_productive,
        }
    }

    /// Checks whether a symbol is productive. Can be used to determine the precise reason
    /// of a rule's unproductiveness.
    pub fn productivity(&self, sym: G::Symbol) -> bool {
        self.productivity[sym.usize()]
    }

    /// Sets symbol reachability. Takes an array of reachable symbols.
    pub fn reachable<Sr>(mut self, syms: Sr) -> Self
        where Sr: AsRef<[G::Symbol]>
    {
        for &sym in syms.as_ref().iter() {
            let reachability = self.reachability[sym.usize()].iter();
            unsafe {
                for (dst, &src) in self.reachable_syms.storage_mut().iter_mut().zip(reachability) {
                    *dst |= src;
                }
            }
        }
        self.all_useful = self.all_productive &
                          self.reachable_syms.storage().iter().all(|&reachable| reachable == !0);
        self
    }

    /// Checks whether all rules in the grammar are useful.
    pub fn all_useful(&self) -> bool {
        self.all_useful
    }

    /// Checks whether all rules in the grammar are productive.
    pub fn all_productive(&self) -> bool {
        self.all_productive
    }
}

// Watch out: Normal type bounds conflict with HRTB.
impl<'a, G> Usefulness<&'a mut G>
    where G: ContextFree,
          &'a G: ContextFreeRef<'a, Target = G>,
          &'a mut G: ContextFreeMut<'a, Target = G>
{
    /// Returns an iterator over the grammar's useless rules.
    pub fn useless_rules(&'a self) -> UselessRules<'a, G, <&'a G as ContextFreeRef<'a>>::Rules> {
        UselessRules {
            rules: self.grammar.rules(),
            usefulness: self,
        }
    }

    /// Removes useless rules. The language represented by the grammar doesn't change.
    pub fn remove_useless_rules(&mut self) {
        if !self.all_useful {
            let productivity = &self.productivity;
            let reachable_syms = &self.reachable_syms;
            self.grammar.retain(|lhs, rhs, _| {
                let productive = rhs.iter().all(|sym| productivity[sym.usize()]);
                let reachable = reachable_syms[lhs.usize()];
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
        if self.usefulness.all_useful {
            return None;
        }

        while let Some(rule) = self.rules.next() {
            let lhs = rule.lhs().usize();
            let productive = rule.rhs().iter().all(|sym| self.usefulness.productivity[sym.usize()]);
            let reachable = self.usefulness.reachable_syms[lhs];

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
