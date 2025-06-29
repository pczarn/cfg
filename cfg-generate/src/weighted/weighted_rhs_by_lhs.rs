use std::collections::BTreeMap;

use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

use super::random::GenRange;
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
        let weighted_rhs_list = self.weights.entry(lhs).or_default();
        weighted_rhs_list.rhs_list.push(WeightedRhs {
            weight: weighted_rhs_list.total_weight,
            rhs: rhs.to_vec(),
        });
        weighted_rhs_list.total_weight += weight;
    }
}

pub trait Weighted {
    fn weighted(&self) -> WeightedRhsByLhs<f64>;
}

impl Weighted for Cfg {
    fn weighted(&self) -> WeightedRhsByLhs<f64> {
        let mut weighted = WeightedRhsByLhs::new();
        for rule in self.rules() {
            let weight = rule.history.weight.unwrap_or(1000) as f64 / 1000f64;
            weighted.add_weight(weight, rule.lhs, &rule.rhs[..]);
        }
        weighted
    }
}

impl<W: Weight> WeightedRhsByLhs<W> {
    pub fn pick_rhs<R>(&self, lhs: Symbol, rng: &mut R) -> &[Symbol]
    where
        R: GenRange,
    {
        if let Some(weighted_rhs_list) = self.weights.get(&lhs) {
            if weighted_rhs_list.rhs_list.len() == 1 {
                return &weighted_rhs_list.rhs_list[0].rhs[..];
            }
            let value = rng.gen(weighted_rhs_list.total_weight.into());
            match weighted_rhs_list.rhs_list.binary_search_by(|weighted_rhs| {
                weighted_rhs
                    .weight
                    .into()
                    .partial_cmp(&value)
                    .expect("invalid float")
            }) {
                Ok(idx) | Err(idx) => &weighted_rhs_list.rhs_list[idx.saturating_sub(1)].rhs[..],
            }
        } else {
            &[]
        }
    }
}
