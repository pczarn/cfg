//! Informs us about recursive rules.

use cfg_grammar::{Cfg, CfgRule};
use cfg_predict_distance::MinimalDistance;
use cfg_symbol_bit_matrix::{CfgSymbolBitMatrixExt, ReachabilityMatrix};

/// Calculation of parts of grammar that participate in recursion,
/// be it left-recursion, right-recursion or middle-recursion.
pub struct Recursion<'a> {
    grammar: &'a Cfg,
    derivation: ReachabilityMatrix,
}

/// Informs us about the kind of recursion in a rule.
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct RecursionKind {
    /// Rule is left-recursive.
    pub left: bool,
    /// Rule has recursion in its middle.
    pub middle: bool,
    /// Rule is right-recursive.
    pub right: bool,
}

/// Refers to a grammar rule alongside information about
/// rule recursion and optionally minimal distance to
/// the closest recursion symbol.
///
/// # Example
///
/// Recursion is transitive. Here, both `A` and `B` are recursive:
///
/// ```ignore
/// A ::= B c
/// B ::= A c | c
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct RecursiveRule<'a> {
    /// Refers to a grammar rule.
    pub rule: &'a CfgRule,
    /// Information about recursion.
    pub recursion: RecursionKind,
    /// Optionally, minimal dsitance to the closest
    /// recursion symbol.
    pub distances: Option<(usize, usize)>,
}

/// Iterator over recursive rules with information about the
/// kind of their recursion.
pub struct RecursiveRules<'a, 'b, R: Iterator<Item = &'b CfgRule>> {
    rules: R,
    recursion: &'b Recursion<'a>,
}

/// Iterator over recursive rules with information about the
/// kind of their recursion as well as symmetric distance to the
/// closest recursion.
pub struct RecursiveRulesWithDistances<'a, 'b, R: Iterator<Item = (usize, &'b CfgRule)>> {
    rules: R,
    recursion: &'b Recursion<'a>,
    minimal_distance: MinimalDistance<'a>,
}

impl<'a> Recursion<'a> {
    /// Returns a new `Recursion` for a grammar.
    pub fn new(grammar: &'a Cfg) -> Self {
        let reachability = grammar.reachability_matrix();

        Recursion {
            grammar: grammar,
            derivation: reachability,
        }
    }

    /// Makes an iterator over rules with information about the
    /// distance to the closest recursion on the RHS.
    ///
    /// The distance is [`Symmetric`], meaning it goes both forwards
    /// and backwars.
    ///
    /// [`Symmetric`]: cfg_predict_distance::DistanceDirection::Symmetric
    pub fn minimal_distances<'b>(&'b self) -> impl Iterator<Item = RecursiveRule<'b>> {
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

    /// Makes an iterator over recursive rules and information
    /// about their recursion.
    pub fn recursive_rules<'b>(&'b self) -> impl Iterator<Item = RecursiveRule<'b>> {
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
                let rule_distances = &self.minimal_distance.distances()[idx][..];
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
    if rule.rhs.len() == 0 {
        return None;
    }
    let rec_kind = RecursionKind {
        left: rule
            .rhs
            .first()
            .map(|&rhs_sym| derivation[(rhs_sym, rule.lhs)])
            == Some(true),
        right: rule
            .rhs
            .last()
            .map(|&rhs_sym| derivation[(rhs_sym, rule.lhs)])
            == Some(true),
        middle: rule
            .rhs
            .iter()
            .skip(1)
            .take(rule.rhs.len().saturating_sub(2))
            .any(|&rhs_sym| derivation[(rhs_sym, rule.lhs)]),
    };
    if rec_kind.any() { Some(rec_kind) } else { None }
}

impl RecursionKind {
    fn any(self) -> bool {
        self.left || self.middle || self.right
    }
}
