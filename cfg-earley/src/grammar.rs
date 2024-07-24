use std::ops::{Deref, DerefMut};

use cfg_grammar::rule_builder::RuleBuilder;
use cfg_grammar::Cfg;
use cfg_history::RootHistoryNode;
use cfg_sequence::builder::SequenceRuleBuilder;
use cfg_sequence::rewrite::SequencesToProductions;
use cfg_symbol::Symbol;

use super::BinarizedGrammar;

/// Drop-in replacement for `cfg::Cfg` that traces relations between user-provided
/// and internal grammars.
#[derive(Default)]
pub struct EarleyGrammar {
    inherit: Cfg,
    start: Option<Symbol>,
}

impl EarleyGrammar {
    pub fn new() -> Self {
        EarleyGrammar {
            inherit: Cfg::new(),
            start: None,
        }
    }

    pub fn set_start(&mut self, start: Symbol) {
        self.start = Some(start);
    }

    pub fn get_start(&self) -> Symbol {
        self.start.unwrap()
    }

    pub fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut Cfg> {
        // TODO: sequence rules?
        let rule_count = self.inherit.rules().count();
        let history_id = self.add_history_node(RootHistoryNode::Origin { origin: rule_count });
        self.inherit.rule(lhs).history(history_id)
    }

    pub fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions<Cfg>> {
        let rule_count = self.inherit.rules().count();
        let history_id = self.add_history_node(RootHistoryNode::Origin { origin: rule_count });
        let sequences_to_productions = SequencesToProductions::new(&mut self.inherit);
        SequenceRuleBuilder::new(sequences_to_productions)
            .sequence(lhs)
            .default_history(history_id)
    }

    pub fn binarize(&mut self) {
        BinarizedGrammar {
            inherit: self.inherit.binarize(),
            start: self.start,
            has_wrapped_start: false,
        }
    }

    pub fn binarize_and_eliminate_nulling(&mut self) -> Self {
        BinarizedGrammar {
            inherit: self.inherit.binarize(),
            start: self.start,
            has_wrapped_start: false,
        }
    }
}

impl Deref for EarleyGrammar {
    type Target = Cfg;
    fn deref(&self) -> &Self::Target {
        &self.inherit
    }
}

impl DerefMut for EarleyGrammar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inherit
    }
}

use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

use bit_matrix::BitMatrix;

use cfg_classify::useful::Usefulness;
use cfg_grammar::history::graph::HistoryGraph;
use cfg_grammar::history::node::RootHistoryNode;
use cfg_grammar::rule::RuleRef;
use cfg_grammar::symbol::remap_symbols::Remap;
use cfg_grammar::{BinarizedCfg, HistoryId, HistoryNode, RuleContainer};
use cfg_symbol::intern::Mapping;
use cfg_symbol::{Symbol, SymbolSource};

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

impl RuleContainer for BinarizedGrammar {
    fn rules<'a>(&'a self) -> impl Iterator<Item = RuleRef<'a>>
    where
        Self: 'a,
    {
        self.inherit.rules()
    }

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
        F: FnMut(RuleRef) -> bool,
    {
        self.inherit.retain(f)
    }

    fn add_rule(&mut self, rule: RuleRef) {
        self.inherit.add_rule(rule);
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
    pub fn original_start(&self) -> Option<Symbol> {
        if !self.has_wrapped_start {
            return None;
        }
        let is_start_rule = |rule: &RuleRef| rule.lhs == self.start();
        let rhs0 = |rule: RuleRef| rule.rhs.get(0).cloned();
        self.rules().find(is_start_rule).and_then(rhs0)
    }

    pub fn eof(&self) -> Option<Symbol> {
        if !self.has_wrapped_start {
            return None;
        }
        let is_start_rule = |rule: &RuleRef| rule.lhs == self.start();
        let rhs1 = |rule: RuleRef| rule.rhs.get(1).cloned();
        self.rules().find(is_start_rule).and_then(rhs1)
    }

    pub fn dot_before_eof(&self) -> Option<Dot> {
        if !self.has_wrapped_start {
            return None;
        }
        let is_start_rule = |rule: RuleRef| rule.lhs == self.start();
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

    pub fn is_empty(&self) -> bool {
        self.rules().all(|rule| Some(rule.lhs) != self.start)
    }
}
