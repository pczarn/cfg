use std::cmp;
use std::cmp::Ordering::{Greater, Less};

use bit_vec::BitVec;

use crate::local_prelude::*;

/// Rhs closure. In some sense, it is a reverse of breadth
/// first search (reverse BFS).
pub struct RhsClosure<'a> {
    inverse_derivation: Vec<Derivation<'a>>,
    work_stack: Vec<Symbol>,
}

struct Derivation<'a> {
    sym: Symbol,
    rule_ref: RuleRef<'a>,
}

impl<'a> Ord for Derivation<'a> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.sym.cmp(&other.sym)
    }
}

impl<'a> PartialOrd for Derivation<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.sym.partial_cmp(&other.sym)
    }
}

impl<'a> Eq for Derivation<'a> {
    fn assert_receiver_is_total_eq(&self) {
        // nothing to do
    }
}

impl<'a> PartialEq for Derivation<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.sym.eq(&other.sym)
    }
}

impl<'a> RhsClosure<'a> {
    /// Records information which is needed to calculate the RHS transitive closure.
    pub fn new(grammar: &'a Cfg) -> Self {
        let mut inverse_derivation = Vec::with_capacity(2 * grammar.rules().size_hint().0);
        for rule in grammar.rules() {
            inverse_derivation.extend(rule.rhs.iter().map(|&sym| Derivation {
                sym,
                rule_ref: rule,
            }));
        }

        inverse_derivation.sort();

        RhsClosure {
            inverse_derivation,
            work_stack: vec![],
        }
    }

    // Calculates the RHS transitive closure.
    pub fn rhs_closure(&mut self, property: &mut BitVec) {
        for (sym_id, sym_has_property) in property.iter().enumerate() {
            if sym_has_property {
                self.work_stack.push(Symbol::from(sym_id));
            }
        }

        let inverse_derivation = &self.inverse_derivation[..];
        while let Some(work_sym) = self.work_stack.pop() {
            for derivation in find(inverse_derivation, work_sym) {
                if !property[derivation.rule_ref.lhs.usize()]
                    && derivation
                        .rule_ref
                        .rhs
                        .iter()
                        .all(|sym| property[sym.usize()])
                {
                    property.set(derivation.rule_ref.lhs.usize(), true);
                    self.work_stack.push(derivation.rule_ref.lhs);
                }
            }
        }
    }
}
