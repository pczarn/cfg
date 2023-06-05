//! Generate random strings from a grammar.

use rand::{thread_rng, Rng};
use crate::{Symbol, symbol::SymbolBitSet};
use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LimitExceeded;

pub trait Random {
    fn random<R: Rng>(&self, limit: Option<u64>, rng: &mut R) -> Result<Vec<Symbol>, LimitExceeded>;

    fn with_thread_rng(&self, limit: Option<u64>) -> Result<Vec<Symbol>, LimitExceeded> {
        let mut thread_rng = thread_rng();
        self.random(limit, &mut thread_rng)
    }
}

impl<W: Weight> Random for WeightedBinarizedGrammar<W> {
    fn random<R: Rng>(&self, limit: Option<u64>, rng: &mut R) -> Result<Vec<Symbol>, LimitExceeded> {
        let weighted = self.weighted();
        let mut work = vec![self.start()];
        let mut result = vec![];
        let terminal_set = SymbolBitSet::terminal_set(self);
        while let Some(sym) = work.pop() {
            if terminal_set.has_sym(sym) {
                result.push(sym);
                if let Some(max_terminals) = limit {
                    if result.len() as u64 > max_terminals {
                        return Err(LimitExceeded);
                    }
                }
            } else {
                let rhs = weighted.pick_rhs(sym, rng);
                work.extend(rhs);
            }
        }
        Ok(result)
    }
}

#[test]
fn test_simplest_random_generation() {
    use crate::ContextFreeRef;
    use super::WeightedGrammar;

    let mut grammar = WeightedGrammar::<u32>::new();
    let (lhs, rhs) = grammar.sym();
    grammar.set_start(lhs);
    grammar.rule(lhs).rhs([rhs]);
    let binarized = grammar.binarize();
    assert_eq!(binarized.num_syms(), 2);
    assert_eq!(binarized.rules().count(), 1);

    let string = binarized.with_thread_rng(Some(1));
    assert_eq!(string, Ok(vec![rhs]));
}
