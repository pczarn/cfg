//! Calculation of minimum distance from one part of the grammar to another.

use std::cmp;

use analysis::RhsClosure;
use grammar::{ContextFree, ContextFreeRef};
use rule::GrammarRule;
use symbol::{Symbol, SymbolBitSet};

/// Calculation of minimum distance from one part of the grammar to another.
/// Similar to multi-source shortest path search in a graph.
pub struct MinimalDistance<'a, G: 'a> {
    grammar: &'a G,
    distances: Vec<Vec<Option<u32>>>,
    prediction_distances: Vec<Option<u32>>,
    completion_distances: Vec<Option<u32>>,
    min_of: Vec<Option<u32>>,
}

impl<'a, G> MinimalDistance<'a, G>
    where G: ContextFree + 'a,
          &'a G: ContextFreeRef<'a, Target = G>
{
    /// Returns a new `MinimalDistance` for a grammar.
    pub fn new(grammar: &'a G) -> Self {
        let distances = grammar.rules().map(|rule| vec![None; rule.rhs().len() + 1]).collect();
        MinimalDistance {
            grammar: grammar,
            distances: distances,
            prediction_distances: vec![None; grammar.num_syms()],
            completion_distances: vec![None; grammar.num_syms()],
            min_of: vec![None; grammar.num_syms()],
        }
    }

    /// Returns distances in order respective to the order of rule iteration.
    pub fn distances(&self) -> &[Vec<Option<u32>>] {
        &self.distances[..]
    }

    /// Calculates minimal distance from one parts of the grammar to others.
    /// Returns distances in order respective to the order of rule iteration.
    pub fn minimal_distances<I, J>(&mut self, iter: I) -> &[Vec<Option<u32>>]
        where I: Iterator<Item=(<&'a G as ContextFreeRef<'a>>::RuleRef, J)>,
              J: Iterator<Item=usize>
    {
        self.minimal_sentence_lengths();
        self.immediate_minimal_distances(iter);
        self.transitive_minimal_distances();
        self.distances()
    }

    fn minimal_sentence_lengths(&mut self) {
        // The distance for terminals is 1.
        let terminal_set = SymbolBitSet::terminal_set(&self.grammar);
        for terminal in terminal_set.iter() {
            self.min_of[terminal.usize()] = Some(1);
        }
        // The distance for nullable symbols is 0.
        for rule in self.grammar.rules() {
            if rule.rhs().is_empty() {
                self.min_of[rule.lhs().usize()] = Some(0);
            }
        }
        // Calculate minimal lengths for nonterminals.
        RhsClosure::new(self.grammar).rhs_closure_with_values(&mut self.min_of);
    }

    fn immediate_minimal_distances<I, J>(&mut self, iter: I)
        where I: Iterator<Item=(<&'a G as ContextFreeRef<'a>>::RuleRef, J)>,
              J: Iterator<Item=usize>
    {
        // Calculates distances within rules.
        for (idx, (rule, positions)) in iter.enumerate() {
            for position in positions {
                let (min, _) = self.update_rule_distances(0, &rule.rhs()[..position], idx);
                set_min(&mut self.prediction_distances[rule.lhs().usize()], min);
            }
        }
    }

    /// Calculates lengths of shortest paths that cross transitions (predictions and completions).
    fn transitive_minimal_distances(&mut self) {
        let mut changed = true;
        while changed {
            // Keep going for as long as any completion distances were lowered in the last pass.
            changed = false;
            for (idx, rule) in self.grammar.rules().enumerate() {
                if let Some(distance) = self.completion_distances[rule.lhs().usize()] {
                    let (_, changed_now) = self.update_rule_distances(distance, rule.rhs(), idx);
                    changed |= changed_now;
                }
            }
        }
    }

    // Update distances in a rule.
    fn update_rule_distances(&mut self, mut cur: u32, rhs: &[Symbol], idx: usize) -> (u32, bool) {
        let set = &mut self.distances[idx];
        for (dot, sym) in rhs.iter().enumerate().rev() {
            set_min(&mut self.completion_distances[sym.usize()], cur);
            set_min(&mut set[dot + 1], cur);
            cur += self.min_of[sym.usize()].unwrap();
            if let Some(sym_predicted) = self.prediction_distances[sym.usize()] {
                cur = cmp::min(cur, sym_predicted);
            }
        }
        let changed = set_min(&mut set[0], cur);
        (cur, changed)
    }
}

/// Updates a value with a minimum of two values.
fn set_min(current: &mut Option<u32>, new: u32) -> bool {
    if let Some(ref mut current) = *current {
        if *current > new {
            *current = new;
            true
        } else {
            false
        }
    } else {
        *current = Some(new);
        true
    }
}
