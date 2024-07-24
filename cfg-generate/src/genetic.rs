#![allow(dead_code)]

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::BTreeSet;
use std::ops::Range;

use cfg_symbol::Symbol;

use self::Rhs::*;
#[cfg(test)]
use self::RuleKind::*;

struct GeneticAlgorithm {
    target: Target,
}

struct Target {
    number_of_symbols: u32,
    population: Population,
}

struct Population {
    rule_descriptions: BTreeSet<RuleDescription>,
}

#[derive(Clone, Eq, PartialEq)]
struct RuleDescription {
    rule_kind: RuleKind,
    rhs: Rhs,
    amount: Range<u32>,
}

impl PartialOrd for RuleDescription {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuleDescription {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = (&self.rule_kind, &self.rhs).cmp(&(&other.rule_kind, &other.rhs));
        ordering
            .then(self.amount.start.cmp(&other.amount.start))
            .then(self.amount.end.cmp(&other.amount.end))
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
enum RuleKind {
    Cyclical,
    LeftRecursive,
    RightRecursive,
    MiddleRecursive { position: u32 },
    Ll { lookahead: u32 },
    Lr { lookahead: u32 },
    Lalr { lookahead: u32 },
    Regular,
    ContextFree,
    AnyKind,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
enum Rhs {
    Unary {
        rhs0: Symbol,
        rhs0_is_transparent: bool,
    },
    Binary {
        rhs0: Symbol,
        rhs0_is_transparent: bool,
        rhs1: Symbol,
        rhs1_is_transparent: bool,
    },
    Any,
}

#[derive(Clone, Copy)]
struct Mutation {
    randomness: u32,
    number_of_symbols: u32,
}

impl Population {
    fn new() -> Self {
        Population {
            rule_descriptions: BTreeSet::new(),
        }
    }

    fn rhs_length(&self) -> u32 {
        unimplemented!()
    }

    fn mutate(&self, mutation: Mutation) -> Self {
        let mut new_population = Population::new();
        for rule_description in &self.rule_descriptions {
            new_population
                .rule_descriptions
                .insert(rule_description.mutate(mutation));
        }
        new_population
    }
}

impl RuleDescription {
    fn mutate(&self, mutation: Mutation) -> Self {
        RuleDescription {
            rule_kind: self.rule_kind.clone(),
            rhs: self.rhs.mutate(mutation),
            amount: self.amount.clone(),
        }
    }
}

impl Rhs {
    fn mutate(&self, _mutation: Mutation) -> Self {
        match self {
            &Unary {
                rhs0,
                rhs0_is_transparent,
            } => Unary {
                rhs0,
                rhs0_is_transparent,
            },
            &Binary {
                rhs0,
                rhs0_is_transparent,
                rhs1,
                rhs1_is_transparent,
            } => Binary {
                rhs0,
                rhs0_is_transparent,
                rhs1,
                rhs1_is_transparent,
            },
            Any => Any,
        }
    }
}

impl Target {
    fn new(number_of_symbols: u32, rule_descriptions: BTreeSet<RuleDescription>) -> Target {
        Target {
            number_of_symbols,
            population: Population { rule_descriptions },
        }
    }
}

#[test]
fn test_zero_mutation() {
    let mut rule_descriptions = BTreeSet::new();
    rule_descriptions.insert(RuleDescription {
        amount: 10..20,
        rule_kind: ContextFree,
        rhs: Any,
    });
    let target = Target::new(10, rule_descriptions.clone());
    let mutation = Mutation {
        randomness: 0,
        number_of_symbols: 10,
    };
    target.population.mutate(mutation);
    for description in &target.population.rule_descriptions {
        assert!(rule_descriptions.contains(description));
    }
}
