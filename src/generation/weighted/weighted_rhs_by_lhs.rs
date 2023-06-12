use std::collections::BTreeMap;

use crate::Symbol;
use super::random::GenRange;
use grammar::ContextFreeRef;
use rule::GrammarRule;

use super::*;

#[derive(Clone, Default, Debug)]
pub struct WeightedRhsByLhs<W> {
    weights: BTreeMap<Symbol, WeightedRhsList<W>>,
}

#[derive(Clone, Default, Debug)]
pub struct WeightedRhs<W> {
    weight: W,
    rhs: Vec<Symbol>,
}

#[derive(Clone, Default, Debug)]
pub struct WeightedRhsList<W> {
    total_weight: W,
    rhs_list: Vec<WeightedRhs<W>>,
}

impl<W: Weight> WeightedRhsByLhs<W> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_weight(&mut self, weight: W, lhs: Symbol, rhs: &[Symbol]) {
        let weighted_rhs_list = self
            .weights
            .entry(lhs)
            .or_insert(WeightedRhsList::default());
        weighted_rhs_list.rhs_list.push(WeightedRhs {
            weight: weighted_rhs_list.total_weight,
            rhs: rhs.to_vec(),
        });
        weighted_rhs_list.total_weight += weight;
    }
}

impl<W: Weight> WeightedGrammar<W> {
    pub fn weighted(&self) -> WeightedRhsByLhs<W> {
        let mut weighted = WeightedRhsByLhs::new();
        for rule in self.rules() {
            weighted.add_weight(rule.history().weight(), rule.lhs(), rule.rhs());
        }
        weighted
    }
}

impl<W: Weight> WeightedBinarizedGrammar<W> {
    pub fn weighted(&self) -> WeightedRhsByLhs<W> {
        let mut weighted = WeightedRhsByLhs::new();
        for rule in self.rules() {
            weighted.add_weight(rule.history().weight(), rule.lhs(), rule.rhs());
        }
        weighted
    }
}

impl<W: Weight> WeightedRhsByLhs<W> {
    #[cfg(feature = "rand")]
    pub fn pick_rhs<R>(&self, lhs: Symbol, rng: &mut R) -> &[Symbol]
    where
        R: GenRange,
    {
        if let Some(weighted_rhs_list) = self.weights.get(&lhs) {
            let value = rng.gen(weighted_rhs_list.total_weight.into());
            match weighted_rhs_list.rhs_list.binary_search_by(|weighted_rhs| {
                weighted_rhs
                    .weight
                    .into()
                    .partial_cmp(&value)
                    .expect("invalid float")
            }) {
                Ok(idx) | Err(idx) => &weighted_rhs_list.rhs_list[idx - 1].rhs[..],
            }
        } else {
            &[]
        }
    }
}
