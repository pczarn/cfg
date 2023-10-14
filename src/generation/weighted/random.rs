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

#[derive(Clone)]
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

impl<I: Iterator<Item = u8> + Clone> GenRange for ByteSource<I> {
    fn gen(&mut self, limit: f64) -> f64 {
        self.0.next().unwrap_or(0) as f64 * limit / 255.0
    }
}

impl<W: Weight> Random for WeightedBinarizedGrammar<W> {
    fn random<R: GenRange + Clone>(
        &self,
        limit: Option<u64>,
        rng: &mut R,
        negative_rules: &[NegativeRules],
        to_char: Fn(Symbol) -> Option<char>,
    ) -> Result<Vec<Symbol>, LimitExceeded> {
        let weighted = self.weighted();
        let mut work = vec![self.start()];
        let mut result = vec![];
        let mut string = String::new();
        let terminal_set = SymbolBitSet::terminal_set(self);
        let negative: BTreeMap<Symbol, &str> = negative_rules.iter().map(|neg| (neg.sym, &neg.chars[..])).collect();
        let mut backtracking: BTreeMap<usize, Vec<(String, R)>> = vec![];
        let mut 
        while let Some(sym) = work.pop() {
            if terminal_set.has_sym(sym) {
                result.push(sym);
                if let Some(ch) = to_char(sym) {
                    string.extend(ch);
                }
                if let Some(max_terminals) = limit {
                    if result.len() as u64 > max_terminals {
                        return Err(LimitExceeded);
                    }
                }
                if let Some((forbidden, backtrack_rng)) = backtracking.get(&string.len()) {
                    if string.ends_with(forbidden) {
                        *rng = backtrack_rng;
                        string.truncate(string.len() - forbidden.len());
                    }
                }
            } else if let Some(forbidden) = negative.get(&sym) {
                backtracking.insert(string.len() + forbidden.len(), (forbidden, rng.clone()));
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
