use std::collections::BTreeMap;

use crate::local_prelude::*;

type RuleIndex = usize;

pub struct OccurenceMap {
    pub occurences: BTreeMap<Symbol, Occurences>,
    empty_occurences: Occurences,
}

/// Two (maybe Small) `Vec`s of rule indices.
#[derive(Clone)]
pub struct Occurences {
    lhs: MaybeSmallVec<RuleIndex>,
    rhs: MaybeSmallVec<RuleIndex>,
}

impl OccurenceMap {
    pub fn from_rules<'a>(rules: impl Iterator<Item = &'a CfgRule>) -> Self {
        let mut occurences = BTreeMap::new();
        for (i, rule) in rules.enumerate() {
            occurences
                .entry(rule.lhs)
                .or_insert(Occurences::new())
                .lhs
                .push(i);
            let mut rhs_syms = rule.rhs.to_vec();
            rhs_syms.sort();
            rhs_syms.dedup();
            for rhs_sym in rhs_syms {
                occurences
                    .entry(rhs_sym)
                    .or_insert(Occurences::new())
                    .rhs
                    .push(i);
            }
        }
        OccurenceMap {
            occurences,
            empty_occurences: Occurences::new(),
        }
    }

    pub fn get(&self, sym: Symbol) -> &Occurences {
        self.occurences.get(&sym).unwrap_or(&self.empty_occurences)
    }
}

impl Occurences {
    fn new() -> Self {
        Occurences {
            lhs: MaybeSmallVec::new(),
            rhs: MaybeSmallVec::new(),
        }
    }

    // pub fn lhs(&self) -> &[RuleIndex] {
    //     &self.lhs[..]
    // }

    pub fn rhs(&self) -> &[RuleIndex] {
        &self.rhs[..]
    }
}
