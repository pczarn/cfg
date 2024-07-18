//! Analysis of rule usefulness.

use bit_matrix::BitMatrix;
use bit_vec::BitVec;

use cfg_grammar::rhs_closure::RhsClosure;
use cfg_grammar::rule::RuleRef;
use cfg_grammar::symbol::symbol_set::SymbolBitSet;
use cfg_grammar::RuleContainer;
use cfg_reachability;
use cfg_symbol::Symbol;

/// Contains the information about usefulness of the grammar's rules.
/// Useful rules are both reachable and productive.
pub struct Usefulness<'a> {
    grammar: &'a Cfg,
    reachability: BitMatrix,
    reachable_syms: SymbolBitSet,
    productivity: SymbolBitSet,
}

/// An iterator over the grammar's useless rules.
pub struct UselessRules<'a, I> {
    usefulness: &'a Usefulness<'a>,
    rules: I,
}

/// A reference to a useless rule, together with the reason for its uselessness.
#[derive(Copy, Clone, Debug)]
pub struct UsefulnessWithRuleRef<'a> {
    rule_ref: RuleRef<'a>,
    /// Indicates whether the rule is unreachable.
    reachable: bool,
    /// Indicates whether the rule is unproductive.
    productive: bool,
}

impl<R> UselessRule<R> {
    pub fn rule(&self) -> &R {
        &self.rule
    }

    pub fn usefulness(&self) -> &RuleUsefulness {
        &self.usefulness
    }
}

impl RuleUsefulness {
    fn is_useless(&self) -> bool {
        !self.reachable || !self.productive
    }
}

/// Returns the set of productive symbols.
fn productive_syms<G>(grammar: &G) -> BitVec
where
    G: RuleContainer,
{
    let mut productive_syms = SymbolBitSet::terminal_or_nulling_set(grammar).into_bit_vec();
    RhsClosure::new(grammar).rhs_closure(&mut productive_syms);
    productive_syms
}

impl<'a, G> Usefulness<&'a mut G>
where
    G: RuleContainer,
{
    /// Analyzes usefulness of the grammar's rules. In particular, it checks for reachable
    /// and productive symbols.
    pub fn new(grammar: &'a mut G) -> Usefulness<&'a mut G> {
        let mut productivity = productive_syms(grammar);
        let reachability = cfg_reachability::reachability_matrix(grammar);
        let unused_syms = SymbolBitSet::new();
        unused_syms.unused(&*grammar);
        let mut reachable_syms = BitVec::from_elem(grammar.sym_source().num_syms(), false);

        productivity.or(&unused_syms);
        reachable_syms.or(&unused_syms);

        debug_assert_eq!(
            reachability.size(),
            (productivity.len(), productivity.len())
        );

        Usefulness {
            grammar,
            productivity,
            reachability,
            reachable_syms,
        }
    }

    /// Checks whether a symbol is productive. Can be used to determine the precise reason
    /// of a rule's unproductiveness.
    pub fn productivity(&self, sym: Symbol) -> bool {
        self.productivity[sym.usize()]
    }

    /// Sets symbol reachability. Takes an array of reachable symbols.
    pub fn reachable<Sr>(mut self, syms: impl AsRef<[Symbol]>) -> Self {
        for &sym in syms.as_ref().iter() {
            let reachability =
                self.reachability[sym.usize()].iter_bits(self.grammar.sym_source().num_syms());
            for (i, is_reachable) in reachability.enumerate() {
                if is_reachable {
                    self.reachable_syms.set(i, true);
                }
            }
        }
        self
    }

    /// Checks whether all rules in the grammar are useful.
    pub fn all_useful(&self) -> bool {
        self.productivity.all() && self.reachable_syms.all()
    }

    /// Checks whether all rules in the grammar are productive.
    pub fn all_productive(&self) -> bool {
        self.productivity.all()
    }

    /// Checks whether all rules in the grammar are reachable.
    pub fn all_reachable(&self) -> bool {
        self.reachable_syms.all()
    }

    pub fn rule_usefulness(&self, rule_ref: RuleRef) -> UsefulnessWithRuleRef<'a> {
        let productive = rule_ref
            .rhs
            .iter()
            .all(|sym| self.productivity[sym.usize()]);
        let reachable = self.reachable_syms[rule_ref.lhs];
        UsefulnessWithRuleRef {
            rule_ref,
            productive,
            reachable,
        }
    }

    /// Returns an iterator over the grammar's useless rules.
    pub fn useless_rules<'b>(&'b self) -> impl Iterator<Item = UsefulnessWithRuleRef<'a>> {
        self.grammar
            .rules()
            .map(|rule_ref| self.rule_usefulness(rule_ref))
            .filter(|useless_rule_ref| useless_rule_ref.is_useless())
    }
}

// Watch out: Normal type bounds conflict with HRTB.
impl<'a> Usefulness<'a> {
    /// Removes useless rules. The language represented by the grammar doesn't change.
    pub fn remove_useless_rules(&mut self) {
        if !self.all_useful() {
            let productivity = &self.productivity;
            let reachable_syms = &self.reachable_syms;
            let rule_is_useful = |rule: RuleRef| {
                let productive = rule.rhs.iter().all(|sym| productivity[sym.usize()]);
                let reachable = reachable_syms[rule.lhs.usize()];
                productive && reachable
            };
            self.grammar.retain(rule_is_useful);
        }
    }
}

impl<'a, G, I> Iterator for UselessRules<'a, G, I>
where
    G: RuleContainer,
    I: Iterator<Item = RuleRef<'a>>,
{
    type Item = UselessRule<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.usefulness.all_useful() {
            return None;
        }

        for rule in &mut self.rules {
            let usefulness = self.usefulness.rule_usefulness(rule);

            if usefulness.is_useless() {
                return Some(UselessRule { rule, usefulness });
            }
        }

        None
    }
}
