//! Calculation of minimum distance from one part of the grammar to another.

use cfg_grammar::*;
use cfg_history::HistoryId;
use cfg_symbol::{Symbol, Symbolic};

/// Calculation of minimum distance from one part of the grammar to another.
/// Similar to multi-source shortest path search in a graph.
pub struct MinimalDistance<'a> {
    grammar: &'a Cfg,
    distances: Vec<(HistoryId, Vec<Option<u32>>)>,
    forward_distances: Distances,
    backward_distances: Distances,
    min_length: Vec<Option<u32>>,
}

struct Distances {
    before: Vec<Option<u32>>,
    after: Vec<Option<u32>>,
}

#[derive(Copy, Clone)]
pub enum DistanceDirection {
    Forward,
    Backward,
    Symmetric,
}

impl DistanceDirection {
    fn includes_forward(self) -> bool {
        match self {
            DistanceDirection::Backward => false,
            _ => true,
        }
    }

    fn includes_backward(self) -> bool {
        match self {
            DistanceDirection::Forward => false,
            _ => true,
        }
    }
}

impl Distances {
    fn new(num_syms: usize) -> Self {
        Distances {
            before: vec![None; num_syms],
            after: vec![None; num_syms],
        }
    }
}

impl<'a> MinimalDistance<'a> {
    /// Returns a new `MinimalDistance` for a grammar.
    pub fn new(grammar: &'a Cfg) -> Self {
        let distances = grammar
            .rules()
            .map(|rule| (rule.history_id, vec![None; rule.rhs.len() + 1]))
            .collect();
        MinimalDistance {
            grammar,
            distances,
            forward_distances: Distances::new(grammar.num_syms()),
            backward_distances: Distances::new(grammar.num_syms()),
            min_length: vec![None; grammar.num_syms()],
        }
    }

    /// Returns distances in order respective to the order of rule iteration.
    pub fn distances(&self) -> &[(HistoryId, Vec<Option<u32>>)] {
        &self.distances[..]
    }

    /// Calculates minimal distance from one parts of the grammar to others.
    /// Returns distances in order respective to the order of rule iteration.
    pub fn minimal_distances(
        &mut self,
        dots: &[(usize, usize)],
        direction: DistanceDirection,
    ) -> &[(HistoryId, Vec<Option<u32>>)] {
        let mut dots = dots.to_vec();
        dots.sort_unstable();
        self.minimal_sentence_lengths();
        self.immediate_minimal_distances(&dots[..], direction);
        self.transitive_minimal_distances();
        self.distances()
    }

    fn minimal_sentence_lengths(&mut self) {
        // The distance for terminals is 1.
        let terminal_set = self.grammar.terminal_symbols();
        for terminal in terminal_set.iter() {
            self.min_length[terminal.usize()] = Some(1);
        }
        // The distance for nullable symbols is 0.
        for rule in self.grammar.rules() {
            if rule.rhs.is_empty() {
                self.min_length[rule.lhs.usize()] = Some(0);
            }
        }
        // Calculate minimal lengths for nonterminals.
        self.grammar
            .clone()
            .rhs_closure_with_values(&mut self.min_length);
    }

    fn immediate_minimal_distances(
        &mut self,
        dots: &[(usize, usize)],
        direction: DistanceDirection,
    ) {
        // Calculates distances within rules.
        for (idx, rule) in self.grammar.rules().enumerate() {
            for (_rule_idx, dot_pos) in binary_search_span(dots, idx) {
                debug_assert_eq!(idx, _rule_idx);
                self.immediate_process_dot(idx, rule, dot_pos, direction);
            }
        }
    }

    fn immediate_process_dot(
        &mut self,
        rule_idx: usize,
        rule: &CfgRule,
        dot_pos: usize,
        direction: DistanceDirection,
    ) {
        if direction.includes_forward() {
            let (min, _) = self.update_rule_distances(0, &rule.rhs[..dot_pos], rule_idx, false);
            set_min(&mut self.forward_distances.before[rule.lhs.usize()], min);
        }
        if direction.includes_backward() {
            let (min, _) = self.update_rule_distances(0, &rule.rhs[dot_pos..], rule_idx, true);
            set_min(&mut self.backward_distances.before[rule.lhs.usize()], min);
        }
    }

    /// Calculates lengths of shortest paths that cross transitions (predictions and completions).
    fn transitive_minimal_distances(&mut self) {
        let mut changed = true;
        while changed {
            // Keep going for as long as any completion distances were lowered in the last pass.
            changed = false;
            for (idx, rule) in self.grammar.rules().enumerate() {
                if let Some(distance) = self.forward_distances.after[rule.lhs.usize()] {
                    let (_, changed_now) =
                        self.update_rule_distances(distance, &rule.rhs[..], idx, false);
                    changed |= changed_now;
                }
                if let Some(distance) = self.backward_distances.after[rule.lhs.usize()] {
                    let (_, changed_now) =
                        self.update_rule_distances(distance, &rule.rhs[..], idx, true);
                    changed |= changed_now;
                }
            }
        }
    }

    // Update distances in a rule.
    fn update_rule_distances(
        &mut self,
        mut cur: u32,
        rhs: &[Symbol],
        idx: usize,
        reverse: bool,
    ) -> (u32, bool) {
        let last = if reverse {
            for (dot, &sym) in rhs.iter().enumerate() {
                set_min(&mut self.distances[idx].1[dot], cur);
                cur = self.update_sym_distance(cur, sym, true);
            }
            rhs.len()
        } else {
            for (dot, &sym) in rhs.iter().enumerate().rev() {
                set_min(&mut self.distances[idx].1[dot + 1], cur);
                cur = self.update_sym_distance(cur, sym, false);
            }
            0
        };
        let changed = set_min(&mut self.distances[idx].1[last], cur);
        (cur, changed)
    }

    fn update_sym_distance(&mut self, mut cur: u32, sym: Symbol, reverse: bool) -> u32 {
        set_min(
            if reverse {
                &mut self.backward_distances.after[sym.usize()]
            } else {
                &mut self.forward_distances.after[sym.usize()]
            },
            cur,
        );
        cur += self.min_length[sym.usize()].unwrap();
        if let Some(sym_predicted) = if reverse {
            &self.backward_distances
        } else {
            &self.forward_distances
        }
        .before[sym.usize()]
        {
            cur.min(sym_predicted)
        } else {
            cur
        }
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

fn binary_search_span(
    dots: &[(usize, usize)],
    idx: usize,
) -> impl Iterator<Item = (usize, usize)> + '_ {
    let dot_idx = match dots.binary_search(&(idx, 0)) {
        Ok(pos) | Err(pos) => pos,
    };
    dots[dot_idx..]
        .iter()
        .copied()
        .take_while(move |&(rule_id, _)| rule_id == idx)
}
