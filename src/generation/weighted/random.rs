//! Generate random strings from a grammar.

use super::*;
use crate::{symbol::SymbolBitSet, Symbol};
use rand::{thread_rng, Rng};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LimitExceeded;

pub trait Random {
    fn random<R: GenRange>(
        &self,
        limit: Option<u64>,
        rng: &mut R,
    ) -> Result<Vec<Symbol>, LimitExceeded>;

    fn with_thread_rng(&self, limit: Option<u64>) -> Result<Vec<Symbol>, LimitExceeded> {
        let mut thread_rng = thread_rng();
        self.random(limit, &mut thread_rng)
    }
}

pub struct ByteSource<I: Iterator<Item = u8>>(I);

pub trait GenRange {
    fn gen(&mut self, limit: f64) -> f64;
}

impl<I: Iterator<Item = u8>> ByteSource<I> {
    pub fn new(iter: I) -> Self {
        ByteSource(iter)
    }
}

impl<R: Rng> GenRange for R {
    fn gen(&mut self, limit: f64) -> f64 {
        self.gen_range(0.0..limit)
    }
}

impl<R: Rng> GenRange for R {
    fn gen(&mut self, limit: f64) -> f64 {
        self.gen_range(0.0..limit)
    }
}

impl<I: Iterator<Item = u8>> GenRange for ByteSource<I> {
    fn gen(&mut self, limit: f64) -> f64 {
        self.0.next().unwrap_or(0) as f64 * limit / 255.0
    }
}

impl<W: Weight> Random for WeightedBinarizedGrammar<W> {
    fn random<R: GenRange>(
        &self,
        limit: Option<u64>,
        rng: &mut R,
    ) -> Result<Vec<Symbol>, LimitExceeded> {
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
                work.extend(rhs.iter().cloned().rev());
            }
        }
        Ok(result)
    }
}

#[test]
fn test_simplest_random_generation() {
    use super::WeightedGrammar;
    use crate::ContextFreeRef;

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
