//! Analysis of rule usefulness.

use bit_vec::BitVec;
use cfg_symbol_bit_matrix::SymbolBitMatrix;

use cfg_grammar::Cfg;
use cfg_grammar::RuleRef;
use cfg_grammar::SymbolBitSet;
use cfg_symbol::Symbol;

/// Contains the information about usefulness of the grammar's rules.
/// Useful rules are both reachable and productive.
pub struct Usefulness<'a> {
    grammar: &'a mut Cfg,
    reachability: SymbolBitMatrix,
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
pub struct UsefulnessForRule<'a> {
    rule_ref: RuleRef<'a>,
    usefulness: RuleUsefulness,
}

#[derive(Copy, Clone, Debug)]
struct RuleUsefulness {
    /// Indicates whether the rule is unreachable.
    reachable: bool,
    /// Indicates whether the rule is unproductive.
    productive: bool,
}

impl<'a> UsefulnessForRule<'a> {
    pub fn rule(&self) -> RuleRef {
        self.rule_ref
    }

    pub fn usefulness(&self) -> RuleUsefulness {
        self.usefulness
    }
}

impl RuleUsefulness {
    fn is_useless(&self) -> bool {
        !self.reachable || !self.productive
    }
}

/// Returns the set of productive symbols.
fn productive_syms(grammar: &mut Cfg) -> SymbolBitSet {
    let mut productive_syms = SymbolBitSet::new();
    productive_syms.terminal(grammar);
    productive_syms.nulling(grammar);
    grammar.rhs_closure(&mut productive_syms);
    productive_syms
}

impl<'a> Usefulness<'a> {
    /// Analyzes usefulness of the grammar's rules. In particular, it checks for reachable
    /// and productive symbols.
    pub fn new(grammar: &'a mut Cfg) -> Self {
        let mut productivity = productive_syms(grammar);
        let reachability = SymbolBitMatrix::reachability_matrix(grammar);
        let mut unused_syms = SymbolBitSet::new();
        unused_syms.used(grammar);
        let mut reachable_syms = SymbolBitSet::from_elem(grammar, false);

        productivity.union(&unused_syms);
        reachable_syms.union(&unused_syms);

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
        self.productivity[sym]
    }

    /// Sets symbol reachability. Takes an array of reachable symbols.
    pub fn reachable(mut self, syms: impl AsRef<[Symbol]>) -> Self {
        for &sym in syms.as_ref() {
            for sym in self.reachability.iter_row_syms(sym) {
                self.reachable_syms.set(sym, true);
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

    pub fn rule_usefulness<'r>(&self, rule_ref: RuleRef<'r>) -> UsefulnessForRule<'r> {
        let productive = rule_ref.rhs.iter().all(|&sym| self.productivity[sym]);
        let reachable = self.reachable_syms[rule_ref.lhs];
        UsefulnessForRule {
            rule_ref,
            usefulness: RuleUsefulness {
                productive,
                reachable,
            },
        }
    }

    /// Returns an iterator over the grammar's useless rules.
    pub fn useless_rules<'s: 'a>(&'s self) -> impl Iterator<Item = UsefulnessForRule<'a>> + 's {
        self.grammar
            .rules()
            .map(|rule_ref| self.rule_usefulness(rule_ref))
            .filter(|useless_rule_usefulness| useless_rule_usefulness.usefulness().is_useless())
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
                let productive = rule.rhs.iter().all(|&sym| productivity[sym]);
                let reachable = reachable_syms[rule.lhs];
                productive && reachable
            };
            self.grammar.retain(rule_is_useful);
        }
    }
}
