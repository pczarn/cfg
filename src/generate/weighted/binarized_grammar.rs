use std::ops::{Deref, DerefMut};

use ContextFree;
use ContextFreeRef;
use BinarizedCfg;
use binarized::BinarizedRules;
use rule::RuleRef;
use rule::container::RuleContainer;
use symbol::SymbolSource;
use Symbol;

use super::*;

/// Drop-in replacement for `cfg::BinarizedCfg`.
#[derive(Clone)]
pub struct WeightedBinarizedGrammar<W> {
    pub(super) inherit: BinarizedCfg<WeightedHistory<W>>,
    pub(super) start: Option<Symbol>,
}

impl<W: Weight> WeightedBinarizedGrammar<W> {
    /// Creates a new `WeightedBinarizedGrammar`.
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

impl<W> Default for WeightedBinarizedGrammar<W> {
    fn default() -> Self {
        WeightedBinarizedGrammar { inherit: BinarizedCfg::new(), start: None }
    }
}

impl<W: Weight> ContextFree for WeightedBinarizedGrammar<W>
{
}

impl<'a, W: Weight> ContextFreeRef<'a> for &'a WeightedBinarizedGrammar<W> {
    type RuleRef = RuleRef<'a, WeightedHistory<W>>;
    type Rules = BinarizedRules<'a, WeightedHistory<W>>;

    fn rules(self) -> Self::Rules {
        self.inherit.rules()
    }
}

impl<W: Weight> RuleContainer for WeightedBinarizedGrammar<W> {
    type History = WeightedHistory<W>;

    fn sym_source(&self) -> &SymbolSource {
        self.inherit.sym_source()
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        self.inherit.sym_source_mut()
    }

    fn retain<F>(&mut self, f: F)
        where F: FnMut(Symbol, &[Symbol], &WeightedHistory<W>) -> bool
    {
        self.inherit.retain(f)
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: WeightedHistory<W>) {
        self.inherit.add_rule(lhs, rhs, history);
    }
}

impl<W> Deref for WeightedBinarizedGrammar<W> {
    type Target = BinarizedCfg<WeightedHistory<W>>;
    fn deref(&self) -> &Self::Target {
        &self.inherit
    }
}

impl<W> DerefMut for WeightedBinarizedGrammar<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inherit
    }
}
