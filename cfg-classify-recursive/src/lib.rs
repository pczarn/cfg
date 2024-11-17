use cfg_grammar::{Cfg, CfgRule};
use cfg_predict_distance::MinimalDistance;
use cfg_symbol_bit_matrix::{CfgSymbolBitMatrixExt, ReachabilityMatrix};

/// Calculation of parts of grammar that participate in recursion,
/// be it left-recursion, right-recursion or middle-recursion.
pub struct Recursion<'a> {
    grammar: &'a Cfg,
    derivation: ReachabilityMatrix,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum RecursionKind {
    Left,
    Right,
    Middle,
    All,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct RecursiveRule<'a> {
    pub rule: &'a CfgRule,
    pub recursion: RecursionKind,
    pub distances: Option<(usize, usize)>,
}

pub struct RecursiveRules<'a, 'b, R: Iterator<Item = &'b CfgRule>> {
    rules: R,
    recursion: &'b Recursion<'a>,
}

pub struct RecursiveRulesWithDistances<'a, 'b, R: Iterator<Item = (usize, &'b CfgRule)>> {
    rules: R,
    recursion: &'b Recursion<'a>,
    minimal_distance: MinimalDistance<'a>,
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

    pub fn minimal_distances<'b>(
        &'b self,
    ) -> RecursiveRulesWithDistances<'a, 'b, impl Iterator<Item = (usize, &'b CfgRule)>> {
        let mut minimal_distance = MinimalDistance::new(&self.grammar);
        let mut dots = vec![];
        for (idx, rule) in self.grammar.rules().enumerate() {
            let recursive_dot_pos = rule
                .rhs
                .iter()
                .enumerate()
                .filter(|&(_dot_pos, &rhs_sym)| self.derivation[(rhs_sym, rule.lhs)]);
            for (dot_pos, _rhs_sym) in recursive_dot_pos {
                dots.push((idx, dot_pos));
            }
        }
        minimal_distance.minimal_distances(
            &dots[..],
            cfg_predict_distance::DistanceDirection::Symmetric,
        );
        RecursiveRulesWithDistances {
            rules: self.grammar.rules().enumerate(),
            recursion: self,
            minimal_distance,
        }
    }

    pub fn recursive_rules<'b>(
        &'b self,
    ) -> RecursiveRules<'a, 'b, impl Iterator<Item = &'b CfgRule>> {
        RecursiveRules {
            rules: self.grammar.rules(),
            recursion: self,
        }
    }
}

impl<'a, 'b, R: Iterator<Item = &'b CfgRule>> Iterator for RecursiveRules<'a, 'b, R> {
    type Item = RecursiveRule<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(rule) = self.rules.next() {
            if let Some(recursion) = rule_recursion(rule, &self.recursion.derivation) {
                return Some(RecursiveRule {
                    rule,
                    recursion,
                    distances: None,
                });
            }
        }
        None
    }
}

impl<'a, 'b, R: Iterator<Item = (usize, &'b CfgRule)>> Iterator
    for RecursiveRulesWithDistances<'a, 'b, R>
{
    type Item = RecursiveRule<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((idx, rule)) = self.rules.next() {
            if let Some(recursion) = rule_recursion(rule, &self.recursion.derivation) {
                // if self.recursion.derivation[(rule.lhs, rule.lhs)] {
                let rule_distances = &self.minimal_distance.distances()[idx].1[..];
                return Some(RecursiveRule {
                    rule,
                    recursion,
                    distances: Some((
                        rule_distances[0].unwrap_or(u32::MAX) as usize,
                        rule_distances[rule_distances.len() - 1].unwrap_or(u32::MAX) as usize,
                    )),
                });
            }
        }
        None
    }
}

fn rule_recursion(rule: &CfgRule, derivation: &ReachabilityMatrix) -> Option<RecursionKind> {
    if rule
        .rhs
        .iter()
        .all(|&rhs_sym| derivation[(rhs_sym, rule.lhs)])
    {
        // ?
        // derivation[(rule.lhs, rule.lhs)]
        return Some(RecursionKind::All);
    }
    if rule
        .rhs
        .iter()
        .skip(1)
        .take(rule.rhs.len().saturating_sub(2))
        .any(|&rhs_sym| derivation[(rhs_sym, rule.lhs)])
    {
        return Some(RecursionKind::Middle);
    }
    if rule
        .rhs
        .first()
        .map(|&rhs_sym| derivation[(rhs_sym, rule.lhs)])
        == Some(true)
    {
        return Some(RecursionKind::Left);
    }
    if rule
        .rhs
        .last()
        .map(|&rhs_sym| derivation[(rhs_sym, rule.lhs)])
        == Some(true)
    {
        return Some(RecursionKind::Right);
    }
    None
}
