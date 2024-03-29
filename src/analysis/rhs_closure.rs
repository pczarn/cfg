use std::cmp::Ordering::{Greater, Less};

use bit_vec::BitVec;

use crate::prelude::*;

/// Rhs closure. In some sense, it is a reverse of breadth
/// first search (reverse BFS).
pub struct RhsClosure<R> {
    inverse_derivation: Vec<(Symbol, R)>,
    work_stack: Vec<Symbol>,
}

impl<R> RhsClosure<R>
where
    R: Copy + GrammarRule,
{
    /// Records information which is needed to calculate the RHS transitive closure.
    pub fn new<'a, G>(grammar: &'a G) -> Self
    where
        G: RuleContainer + Default,
        &'a G: RuleContainerRef<'a, Target = G, RuleRef = R>,
        R: 'a,
    {
        let mut inverse_derivation = Vec::with_capacity(2 * grammar.rules().size_hint().0);
        for rule in grammar.rules() {
            inverse_derivation.extend(rule.rhs().iter().map(|&sym| (sym, rule)));
        }

        inverse_derivation.sort_by(|&(ref sym_a, _), &(ref sym_b, _)| sym_a.cmp(&sym_b));

        RhsClosure {
            inverse_derivation: inverse_derivation,
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
            for &(_, rule) in find(inverse_derivation, work_sym) {
                if !property[rule.lhs().usize()]
                    && rule.rhs().iter().all(|sym| property[sym.usize()])
                {
                    property.set(rule.lhs().usize(), true);
                    self.work_stack.push(rule.lhs());
                }
            }
        }
    }

    // Calculates the RHS transitive closure.
    pub fn rhs_closure_for_any(&mut self, property: &mut BitVec) {
        for (sym_id, sym_has_property) in property.iter().enumerate() {
            if sym_has_property {
                self.work_stack.push(Symbol::from(sym_id));
            }
        }

        let inverse_derivation = &self.inverse_derivation[..];
        while let Some(work_sym) = self.work_stack.pop() {
            for &(_, rule) in find(inverse_derivation, work_sym) {
                if !property[rule.lhs().usize()]
                    && rule.rhs().iter().any(|sym| property[sym.usize()])
                {
                    property.set(rule.lhs().usize(), true);
                    self.work_stack.push(rule.lhs());
                }
            }
        }
    }

    // Calculates the RHS transitive closure.
    pub fn rhs_closure_with_values(&mut self, value: &mut Vec<Option<u32>>) {
        for (sym_id, maybe_sym_value) in value.iter().enumerate() {
            if maybe_sym_value.is_some() {
                self.work_stack.push(Symbol::from(sym_id));
            }
        }

        let inverse_derivation = &self.inverse_derivation[..];
        while let Some(work_sym) = self.work_stack.pop() {
            for &(_, rule) in find(inverse_derivation, work_sym) {
                let maybe_work_value = rule.rhs().iter().fold(Some(0), |acc, elem| {
                    let elem_value = value[elem.usize()];
                    if let (Some(a), Some(b)) = (acc, elem_value) {
                        Some(a + b)
                    } else {
                        None
                    }
                });
                if let Some(work_value) = maybe_work_value {
                    if let Some(current_value) = value[rule.lhs().usize()] {
                        if current_value <= work_value {
                            continue;
                        }
                    }
                    value[rule.lhs().usize()] = Some(work_value);
                    self.work_stack.push(rule.lhs());
                }
            }
        }
    }
}

fn find<S, R>(inverse_derivation: &[(S, R)], key_sym: S) -> &[(S, R)]
where
    S: Copy + Ord,
{
    match inverse_derivation.binary_search_by(|&(sym, _)| (sym, Greater).cmp(&(key_sym, Less))) {
        Err(idx) => {
            let len = inverse_derivation[idx..]
                .iter()
                .take_while(|t| t.0 == key_sym)
                .count();
            &inverse_derivation[idx..idx + len]
        }
        Ok(_) => unreachable!(),
    }
}
