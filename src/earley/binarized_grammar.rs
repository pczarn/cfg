use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

use bit_matrix::BitMatrix;

use crate::binarized::BinarizedRules;
use crate::classification::useful::Usefulness;
use crate::history::{HistoryGraph, RootHistoryNode};
use crate::prelude::*;
use crate::remap::{Mapping, Remap};
use crate::rule::GrammarRule;
use crate::rule::RuleRef;

type Dot = u32;

/// Drop-in replacement for `cfg::BinarizedCfg`.
#[derive(Clone, Default)]
pub struct BinarizedGrammar {
    pub(super) inherit: BinarizedCfg,
    pub(super) start: Option<Symbol>,
    pub(super) has_wrapped_start: bool,
}

impl BinarizedGrammar {
    /// Creates a new `BinarizedGrammar`.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_start(&mut self, start: Symbol) {
        self.start = Some(start);
    }

    pub fn start(&self) -> Symbol {
        self.start.unwrap()
    }
}

impl<'a> RuleContainerRef<'a> for &'a BinarizedGrammar {
    type RuleRef = RuleRef<'a>;
    type Rules = BinarizedRules<'a>;

    fn rules(self) -> Self::Rules {
        self.inherit.rules()
    }
}

impl RuleContainer for BinarizedGrammar {
    fn history_graph(&self) -> &HistoryGraph {
        self.inherit.history_graph()
    }

    fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        self.inherit.add_history_node(node)
    }

    fn sym_source(&self) -> &SymbolSource {
        self.inherit.sym_source()
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        self.inherit.sym_source_mut()
    }

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(Symbol, &[Symbol], HistoryId) -> bool,
    {
        self.inherit.retain(f)
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: HistoryId) {
        self.inherit.add_rule(lhs, rhs, history);
    }
}

impl Deref for BinarizedGrammar {
    type Target = BinarizedCfg;
    fn deref(&self) -> &Self::Target {
        &self.inherit
    }
}

impl DerefMut for BinarizedGrammar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inherit
    }
}

impl BinarizedGrammar {
    pub fn wrap_start(&mut self) {
        let start = self.start();
        let (new_start, eof) = self.sym();
        let history = self.add_history_node(RootHistoryNode::NoOp.into());
        self.add_rule(new_start, &[start, eof], history);
        self.set_start(new_start);
        self.has_wrapped_start = true;
    }

    pub fn original_start(&self) -> Option<Symbol> {
        if !self.has_wrapped_start {
            return None;
        }
        let is_start_rule = |rule: &RuleRef| rule.lhs() == self.start();
        let rhs0 = |rule: RuleRef| rule.rhs().get(0).cloned();
        self.rules().find(is_start_rule).and_then(rhs0)
    }

    pub fn eof(&self) -> Option<Symbol> {
        if !self.has_wrapped_start {
            return None;
        }
        let is_start_rule = |rule: &RuleRef| rule.lhs() == self.start();
        let rhs1 = |rule: RuleRef| rule.rhs().get(1).cloned();
        self.rules().find(is_start_rule).and_then(rhs1)
    }

    pub fn dot_before_eof(&self) -> Option<Dot> {
        if !self.has_wrapped_start {
            return None;
        }
        let is_start_rule = |rule: RuleRef| rule.lhs() == self.start();
        let as_dot = |pos| pos as Dot;
        self.rules().position(is_start_rule).map(as_dot)
    }

    pub fn make_proper(mut self: BinarizedGrammar) -> BinarizedGrammar {
        let start = self.start();
        {
            let mut usefulness = Usefulness::new(&mut *self).reachable([start]);
            if !usefulness.all_useful() {
                // for useless in usefulness.useless_rules() {
                //     let rhs: Vec<_> = useless.rule.rhs().iter().map(|t| tok.get(t.usize())).collect();
                //     println!("lhs:{:?} rhs:{:?} unreachable:{} unproductive:{}", tok.get(useless.rule.lhs().usize()), rhs, useless.unreachable, useless.unproductive);
                // }
                println!("warning: grammar has useless rules");
                usefulness.remove_useless_rules();
            }
        };
        self
    }

    pub fn eliminate_nulling(mut self: BinarizedGrammar) -> (BinarizedGrammar, BinarizedGrammar) {
        let nulling_grammar = BinarizedGrammar {
            inherit: self.eliminate_nulling_rules(),
            start: Some(self.start()),
            has_wrapped_start: self.has_wrapped_start,
        };
        (self, nulling_grammar)
    }

    pub fn remap_symbols(mut self: BinarizedGrammar) -> (BinarizedGrammar, Mapping) {
        let num_syms = self.sym_source().num_syms();
        // `order` describes relation `A < B`.
        let mut order = BitMatrix::new(num_syms, num_syms);
        for rule in self.rules() {
            if rule.rhs().len() == 1 {
                let left = rule.lhs().usize();
                let right = rule.rhs()[0].usize();
                match left.cmp(&right) {
                    Ordering::Less => {
                        order.set(left, right, true);
                    }
                    Ordering::Greater => {
                        order.set(right, left, true);
                    }
                    Ordering::Equal => {}
                }
            }
        }
        // the order above is not transitive.
        // We modify it so that if `A < B` and `B < C` then `A < C`
        order.transitive_closure();
        let mut maps = {
            let mut remap = Remap::new(&mut *self);
            remap.remove_unused_symbols();
            remap.reorder_symbols(|left, right| {
                let (left, right) = (left.usize(), right.usize());
                if order[(left, right)] {
                    Ordering::Less
                } else if order[(right, left)] {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });
            remap.get_mapping()
        };
        let start = self.start();
        if let Some(internal_start) = maps.to_internal[start.usize()] {
            self.set_start(internal_start);
        } else {
            // The trivial grammar is a unique edge case -- the start symbol was removed.
            let internal_start = Symbol::from(maps.to_external.len());
            maps.to_internal[start.usize()] = Some(internal_start);
            maps.to_external.push(start);
            self.set_start(internal_start);
        }
        (self, maps)
    }

    pub fn is_empty(&self) -> bool {
        self.rules().all(|rule| Some(rule.lhs()) != self.start)
    }
}
