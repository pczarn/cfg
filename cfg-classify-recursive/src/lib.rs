use cfg_grammar::{Cfg, CfgRule};
use cfg_predict_distance::MinimalDistance;
use cfg_symbol_bit_matrix::{CfgSymbolBitMatrixExt, ReachabilityMatrix};

/// Calculation of parts of grammar that participate in recursion,
/// be it left-recursion, right-recursion or middle-recursion.
pub struct Recursion<'a> {
    grammar: &'a Cfg,
    derivation: ReachabilityMatrix,
}

pub enum RecursionKind {
    Left,
    Right,
    Middle,
    All,
}

pub struct RecursiveRule {
    pub rule: CfgRule,
    pub recursion: RecursionKind,
}

pub struct RecursiveRules<'a, 'b, R: Iterator<Item = &'b CfgRule>> {
    rules: R,
    recursion: &'b Recursion<'a>,
    filter: RecursionKind,
}

pub struct RecursiveRuleDistances<'a, 'b, R: Iterator<Item = (usize, &'b CfgRule)>> {
    rules: R,
    recursion: &'b Recursion<'a>,
    minimal_distance: MinimalDistance<'a>,
}

pub struct CfgRuleWithDistance {
    pub rule: CfgRule,
    pub prediction_distance: usize,
    pub completion_distance: usize,
}

impl<'a> Recursion<'a> {
    /// Returns a new `MinimalDistance` for a grammar.
    pub fn new(grammar: &'a Cfg) -> Self {
        let reachability = grammar.reachability_matrix();

        Recursion {
            grammar: grammar,
            derivation: reachability,
        }
    }

    pub fn minimal_distances<'b>(&'b self) -> RecursiveRuleDistances<'a, 'b, impl Iterator<Item = (usize, &'b CfgRule)>> {
        let mut minimal_distance = MinimalDistance::new(&self.grammar);
        let mut dots = vec![];
        for (idx, rule) in self.grammar.rules().enumerate() {
            let recursive_dot_pos = rule.rhs.iter().enumerate().filter(|&(_dot_pos, &rhs_sym)| self.derivation[(rhs_sym, rule.lhs)]);
            for (dot_pos, _rhs_sym) in recursive_dot_pos {
                dots.push((idx, dot_pos));
            }
        }
        minimal_distance.minimal_distances(&dots[..]);
        RecursiveRuleDistances { rules: self.grammar.rules().enumerate(), recursion: self, minimal_distance }
    }

    pub fn recursive_rules<'b>(
        &'b self,
    ) -> RecursiveRules<'a, 'b, impl Iterator<Item = &'b CfgRule>> {
        RecursiveRules {
            rules: self.grammar.rules(),
            recursion: self,
            filter: RecursionKind::All,
        }
    }
}

impl<'a, 'b, R: Iterator<Item = &'b CfgRule>> Iterator for RecursiveRules<'a, 'b, R> {
    type Item = &'b CfgRule;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(rule) = self.rules.next() {
            if rule.rhs.iter().any(|&rhs_sym| self.recursion.derivation[(rhs_sym, rule.lhs)]) {
            // if self.recursion.derivation[(rule.lhs, rule.lhs)] {
                return Some(rule);
            }
        }
        None
    }
}

impl<'a, 'b, R: Iterator<Item = (usize, &'b CfgRule)>> Iterator for RecursiveRuleDistances<'a, 'b, R> {
    type Item = CfgRuleWithDistance;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((idx, rule)) = self.rules.next() {
            if rule.rhs.iter().any(|&rhs_sym| self.recursion.derivation[(rhs_sym, rule.lhs)]) {
            // if self.recursion.derivation[(rule.lhs, rule.lhs)] {
                let rule_distances = &self.minimal_distance.distances()[idx].1[..];
                return Some(CfgRuleWithDistance {
                    rule: rule.clone(),
                    prediction_distance: rule_distances[0].unwrap_or(u32::MAX) as usize,
                    completion_distance: rule_distances[rule_distances.len() - 1].unwrap_or(u32::MAX) as usize,
                });
            }
        }
        None
    }
}
