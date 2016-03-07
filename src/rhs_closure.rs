use std::cmp::Ordering::{Less, Greater};

use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef};
use rule::GrammarRule;
use symbol::Symbol;

pub struct RhsClosure<R> {
    derived_by: Vec<(Symbol, R)>,
    work_stack: Vec<Symbol>,
}

impl<R> RhsClosure<R>
    where R: Copy + GrammarRule
{
    /// Records information which is needed to calculate the RHS transitive closure.
    pub fn new<'a, G>(grammar: &'a G) -> Self
        where G: ContextFree<History = R::History>,
              &'a G: ContextFreeRef<'a, RuleRef = R, Target = G>,
              R: 'a
    {
        let mut derived_by = Vec::with_capacity(2 * grammar.rules().size_hint().0);
        for rule in grammar.rules() {
            derived_by.extend(rule.rhs().iter().map(|&sym| (sym, rule)));
        }

        derived_by.sort_by(|&(ref sym_a, _), &(ref sym_b, _)| sym_a.cmp(&sym_b));

        RhsClosure {
            derived_by: derived_by,
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

        let derived_by = &self.derived_by;
        while let Some(work_sym) = self.work_stack.pop() {
            for &(_, rule) in find(derived_by, work_sym) {
                if !property[rule.lhs().usize()] &&
                   rule.rhs().iter().all(|sym| property[sym.usize()]) {
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

        let derived_by = &self.derived_by;
        while let Some(work_sym) = self.work_stack.pop() {
            for &(_, rule) in find(derived_by, work_sym) {
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

fn find<S, R>(vec: &Vec<(S, R)>, key_sym: S) -> &[(S, R)]
    where S: Copy + Ord
{
    match vec.binary_search_by(|&(sym, _)| (sym, Greater).cmp(&(key_sym, Less))) {
        Err(idx) => {
            let len = vec[idx..].iter().take_while(|t| t.0 == key_sym).count();
            &vec[idx..idx + len]
        }
        Ok(_) => unreachable!(),
    }
}
