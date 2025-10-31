//! Generate random strings from a grammar.

use std::collections::BTreeMap;

use cfg_grammar::Cfg;
use cfg_symbol::Symbol;
// use log::debug;
use rpds::List;

use cfg_grammar::symbol_bit_set::SymbolBitSet;
use rand::rngs::ThreadRng;
use rand::{Rng, rng};

use crate::weighted::weighted_rhs_by_lhs::Weighted;

/// Returned when we fail to generate a random output.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RandomGenError {
    /// We produced a number of terminal symbols above the
    /// given limit.
    LimitExceeded,
    /// Too many attempts to reverse progress after we encountered
    /// output that
    NegativeRuleAttemptsExceeded,
}

/// Limits for generation.
#[derive(Copy, Clone)]
pub struct Limits {
    /// How many terminals we are allowed to produce.
    pub terminals: u64,
    ///
    pub negative_rule_attempts: u64,
}

/// Extension trait that allows generation.
pub trait Random {
    /// Generates symbols derivable from the **start** symbol, driven
    /// by the given **rng**. Respects provided limits.
    ///  
    ///
    /// Computes a list of terminal symbols as well as a `String`, which
    /// is included in the language described by this grammar.
    ///
    /// # Errors
    ///
    /// Returns an error when limits are exceeded.
    fn random<R: GenRange + Clone, F: Fn(Symbol, &mut R) -> Option<char>>(
        &self,
        start: Symbol,
        limit: Option<Limits>,
        rng: &mut R,
        negative_rules: &[NegativeRule],
        to_char: F,
    ) -> Result<(Vec<Symbol>, Vec<char>), RandomGenError>;

    /// Same as `fn random`, but initializes a thread rng
    /// and uses it to produce output.
    fn with_thread_rng<F>(
        &self,
        start: Symbol,
        limit: Option<Limits>,
        negative_rules: &[NegativeRule],
        to_char: F,
    ) -> Result<(Vec<Symbol>, Vec<char>), RandomGenError>
    where
        F: Fn(Symbol, &mut ThreadRng) -> Option<char>,
    {
        let mut thread_rng = rng();
        self.random(start, limit, &mut thread_rng, negative_rules, to_char)
    }
}

/// Can be used in place of the `Rng` to generate in a determinstic
/// way.
#[derive(Clone)]
pub struct ByteSource<I: Iterator<Item = u8>>(I, Vec<u8>);

/// Extension trait for [`Rng`]s.
pub trait GenRange {
    /// Generates a random floating point value
    /// up to the given `limit`.
    fn generate(&mut self, limit: f64) -> f64;

    /// Allows us to skip some random values.
    fn mutate_start(&mut self, attempt_number: u64);
}

impl<I: Iterator<Item = u8>> ByteSource<I> {
    /// Creates an em
    pub fn new(iter: I) -> Self {
        ByteSource(iter, vec![])
    }
}

impl<R: Rng> GenRange for R {
    fn generate(&mut self, limit: f64) -> f64 {
        self.random_range(0.0..limit)
    }

    fn mutate_start(&mut self, attempt_number: u64) {
        for _ in 0..=attempt_number {
            let _: u8 = self.random();
        }
    }
}

impl<I: Iterator<Item = u8> + Clone> GenRange for ByteSource<I> {
    fn generate(&mut self, limit: f64) -> f64 {
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

/// For negative lookahead.
#[derive(Copy, Clone)]
pub struct NegativeRule {
    /// The symbol that, when detected in the output we generate,
    /// that place cannot match `chars` because we will backtrack.
    pub sym: Symbol,
    /// The forbidden characters.
    pub chars: &'static str,
}

struct BacktrackState<'a, R> {
    forbidden: &'a [char],
    rng: R,
    result_len: usize,
    prev_work: List<Symbol>,
}

impl Random for Cfg {
    fn random<R: GenRange + Clone, F: Fn(Symbol, &mut R) -> Option<char>>(
        &self,
        start: Symbol,
        limit: Option<Limits>,
        rng: &mut R,
        negative_rules: &[NegativeRule],
        to_char: F,
    ) -> Result<(Vec<Symbol>, Vec<char>), RandomGenError> {
        let weighted = self.weighted();
        let mut work = List::new();
        work.push_front_mut(start);
        let mut result = vec![];
        let mut string = vec![];
        let mut terminal_set = SymbolBitSet::new();
        terminal_set.terminal(self);
        let negative: BTreeMap<Symbol, Vec<char>> = negative_rules
            .iter()
            .map(|neg| (neg.sym, neg.chars.chars().collect()))
            .collect();
        let mut backtracking: BTreeMap<usize, Vec<BacktrackState<R>>> = BTreeMap::new();
        let mut backtracking_attempts: BTreeMap<usize, u64> = BTreeMap::new();
        while let Some(&sym) = work.first() {
            work.drop_first_mut();
            if terminal_set[sym] {
                result.push(sym);
                if let Some(ch) = to_char(sym, rng) {
                    string.push(ch);
                }
                if let Some(our_limits) = limit {
                    if result.len() as u64 > our_limits.terminals {
                        return Err(RandomGenError::LimitExceeded);
                    }
                }
                if let Some(back) = backtracking.get_mut(&string.len()) {
                    for state in back.iter_mut() {
                        if string.ends_with(state.forbidden) {
                            *rng = state.rng.clone();
                            string.truncate(string.len() - state.forbidden.len());
                            result.truncate(state.result_len);
                            work = state.prev_work.clone();
                            let attempts = backtracking_attempts
                                .get_mut(&string.len())
                                .expect("bt.attempt not found");
                            rng.mutate_start(*attempts);
                            *attempts += 1;
                            if *attempts > 256 * 64 {
                                return Err(RandomGenError::NegativeRuleAttemptsExceeded);
                            }
                        }
                    }
                }
            } else if let Some(forbidden) = negative.get(&sym) {
                // debug!("NEGATIVE: forbidden {:?} at {:?}", forbidden, string.len());
                backtracking
                    .entry(string.len() + forbidden.len())
                    .or_default()
                    .push(BacktrackState {
                        forbidden: &forbidden[..],
                        rng: rng.clone(),
                        result_len: result.len(),
                        prev_work: work.clone(),
                    });
                backtracking_attempts.entry(string.len()).or_insert(0);
            } else {
                let rhs = weighted.pick_rhs(sym, rng);
                // debug!("PICK RHS: from {:?} at {:?}", rhs, string.len());
                for sym in rhs.iter().cloned().rev() {
                    work.push_front_mut(sym);
                }
            }
        }
        Ok((result, string))
    }
}

#[test]
fn test_simplest_random_generation() {
    use cfg_grammar::Cfg;

    let mut grammar = Cfg::new();
    let [lhs, rhs] = grammar.sym();
    grammar.rule(lhs).rhs([rhs]);
    grammar.limit_rhs_len(Some(2));
    assert_eq!(grammar.num_syms(), 2);
    assert_eq!(grammar.rules().count(), 1);

    let to_char = |sym, _: &mut _| if sym == rhs { Some('X') } else { None };
    let string = grammar.with_thread_rng(
        lhs,
        Some(Limits {
            terminals: 1,
            negative_rule_attempts: 0,
        }),
        &[],
        to_char,
    );
    assert_eq!(string, Ok((vec![rhs], vec!['X'])));
}
