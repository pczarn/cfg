//! Generate random strings from a grammar.

use std::collections::BTreeMap;

use rpds::List;

use crate::{prelude::*, symbol::SymbolBitSet};
use rand::{thread_rng, Rng, rngs::ThreadRng};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RandomGenError {
    LimitExceeded,
    NegativeRuleAttemptsExceeded,
}

pub trait Random {
    fn random<R: GenRange + Clone, F: Fn(Symbol, &mut R) -> Option<char>>(
        &self,
        start: Symbol,
        limit: Option<u64>,
        rng: &mut R,
        negative_rules: &[NegativeRule],
        to_char: F,
    ) -> Result<Vec<Symbol>, RandomGenError>;

    fn with_thread_rng<F: Fn(Symbol, &mut ThreadRng) -> Option<char>>(
        &self,
        start: Symbol,
        limit: Option<u64>,
        negative_rules: &[NegativeRule],
        to_char: F,
    ) -> Result<Vec<Symbol>, RandomGenError> {
        let mut thread_rng = thread_rng();
        self.random(start, limit, &mut thread_rng, negative_rules, to_char)
    }
}

#[derive(Clone)]
pub struct ByteSource<I: Iterator<Item = u8>>(I, Vec<u8>);

pub trait GenRange {
    fn gen(&mut self, limit: f64) -> f64;

    fn mutate_start(&mut self, attempt_number: u64);
}

impl<I: Iterator<Item = u8>> ByteSource<I> {
    pub fn new(iter: I) -> Self {
        ByteSource(iter, vec![])
    }
}

impl<R: Rng> GenRange for R {
    fn gen(&mut self, limit: f64) -> f64 {
        self.gen_range(0.0..limit)
    }

    fn mutate_start(&mut self, attempt_number: u64) {
        for _ in 0..=attempt_number {
            let _: u8 = self.gen();
        }
    }
}

impl<I: Iterator<Item = u8> + Clone> GenRange for ByteSource<I> {
    fn gen(&mut self, limit: f64) -> f64 {
        let byte = match self.1.pop() {
            Some(ahead) => ahead,
            None => self.0.next().unwrap_or(0),
        };
        byte as f64 * limit / 255.0
    }

    fn mutate_start(&mut self, attempt_number: u64) {
        fn mix(byte: &mut u8) {
            *byte ^= *byte >> 5;
            *byte = byte.wrapping_mul(123);
            *byte ^= *byte >> 5;
            *byte = byte.wrapping_mul(34);
            *byte ^= *byte >> 5;
        }
        let mut result = vec![];
        for _ in 0..=attempt_number / 256 {
            let mut byte = self.0.next().unwrap_or(0);
            byte ^= attempt_number as u8;
            mix(&mut byte);
            result.push(byte);
        }
        result.reverse();
        self.1.extend(result);
    }
}

#[derive(Copy, Clone)]
pub struct NegativeRule {
    pub sym: Symbol,
    pub chars: &'static str,
}

struct BacktrackState<'a, R> {
    forbidden: &'a str,
    rng: R,
    attempts: u64,
    result_len: usize,
    prev_work: List<Symbol>,
}

impl Random for BinarizedCfg {
    fn random<R: GenRange + Clone, F: Fn(Symbol, &mut R) -> Option<char>>(
        &self,
        start: Symbol,
        limit: Option<u64>,
        rng: &mut R,
        negative_rules: &[NegativeRule],
        to_char: F,
    ) -> Result<Vec<Symbol>, RandomGenError> {
        let weighted = self.weighted();
        let mut work = List::new();
        work.push_front_mut(start);
        let mut result = vec![];
        let mut string = String::new();
        let terminal_set = SymbolBitSet::terminal_set(self);
        let negative: BTreeMap<Symbol, &str> = negative_rules
            .iter()
            .map(|neg| (neg.sym, &neg.chars[..]))
            .collect();
        let mut backtracking: BTreeMap<usize, Vec<BacktrackState<R>>> =
            BTreeMap::new();
        // let mut
        while let Some(&sym) = work.first() {
            work.drop_first_mut();
            if terminal_set.has_sym(sym) {
                result.push(sym);
                if let Some(ch) = to_char(sym, rng) {
                    string.extend(::std::iter::once(ch));
                }
                if let Some(max_terminals) = limit {
                    if result.len() as u64 > max_terminals {
                        return Err(RandomGenError::LimitExceeded);
                    }
                }
                if let Some(back) = backtracking.get_mut(&string.len()) {
                    for state in back.iter_mut()
                    {
                        if string.ends_with(state.forbidden) {
                            *rng = state.rng.clone();
                            string.truncate(string.len() - state.forbidden.len());
                            result.truncate(state.result_len);
                            work = state.prev_work.clone();
                            rng.mutate_start(state.attempts);
                            state.attempts += 1;
                            if state.attempts > 1024 * 1024 * 32 {
                                return Err(RandomGenError::NegativeRuleAttemptsExceeded);
                            }
                        }
                    }
                }
            } else if let Some(&forbidden) = negative.get(&sym) {
                backtracking
                    .entry(string.len() + forbidden.len())
                    .or_insert(vec![])
                    .push(BacktrackState {
                        forbidden: forbidden,
                        rng: rng.clone(),
                        attempts: 0,
                        result_len: result.len(),
                        prev_work: work.clone()
                    });
            } else {
                let rhs = weighted.pick_rhs(sym, rng);
                for sym in rhs.iter().cloned().rev() {
                    work.push_front_mut(sym);
                }
            }
        }
        Ok(result)
    }
}

#[test]
fn test_simplest_random_generation() {
    use crate::prelude::*;

    let mut grammar = Cfg::new();
    let (lhs, rhs) = grammar.sym();
    grammar.rule(lhs).rhs([rhs]);
    let binarized = grammar.binarize();
    assert_eq!(binarized.num_syms(), 2);
    assert_eq!(binarized.rules().count(), 1);

    let string = binarized.with_thread_rng(lhs, Some(1), &[], |_, _| None);
    assert_eq!(string, Ok(vec![rhs]));
}
