use std::cmp::Ordering::{Less, Greater};

use bit_vec::BitVec;

use grammar::{ContextFree, ContextFreeRef};
use rule::GrammarRule;
use symbol::GrammarSymbol;

pub struct RhsClosure<G, R>
    where G: ContextFree
{
    derived_by: Vec<(G::Symbol, R)>,
    work_stack: Vec<G::Symbol>,
}

impl<G, R> RhsClosure<G, R>
    where G: ContextFree,
          R: Copy + GrammarRule<History = G::History, Symbol = G::Symbol>
{
    /// Records information which is needed to calculate the RHS transitive closure.
    pub fn new<'a>(grammar: &'a G) -> Self
        where &'a G: ContextFreeRef<'a, RuleRef = R, Target = G>,
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
                self.work_stack.push(G::Symbol::from(sym_id as u64));
            }
        }

        let derived_by = &self.derived_by;
        while let Some(work_sym) = self.work_stack.pop() {
            match derived_by.binary_search_by(|&(sym, _)| (sym, Greater).cmp(&(work_sym, Less))) {
                Err(idx) => {
                    for &(_, rule) in derived_by[idx..].iter().take_while(|t| t.0 == work_sym) {
                        if !property[rule.lhs().usize()] &&
                           rule.rhs().iter().all(|sym| property[sym.usize()]) {
                            property.set(rule.lhs().usize(), true);
                            self.work_stack.push(rule.lhs());
                        }
                    }
                }
                Ok(_) => unreachable!(),
            }
        }
    }
}
