use std::ops::{Deref, DerefMut};

use cfg_grammar::history::node::RootHistoryNode;
use cfg_grammar::rule::builder::RuleBuilder;
use cfg_grammar::{Cfg, RuleContainer};
use cfg_sequence::builder::SequenceRuleBuilder;
use cfg_sequence::rewrite::SequencesToProductions;
use cfg_symbol::Symbol;

use super::BinarizedGrammar;

/// Drop-in replacement for `cfg::Cfg` that traces relations between user-provided
/// and internal grammars.
#[derive(Default)]
pub struct Grammar {
    inherit: Cfg,
    start: Option<Symbol>,
}

impl Grammar {
    pub fn new() -> Self {
        Grammar {
            inherit: Cfg::new(),
            start: None,
        }
    }

    pub fn set_start(&mut self, start: Symbol) {
        self.start = Some(start);
    }

    pub fn start(&self) -> Symbol {
        self.start.unwrap()
    }

    pub fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut Cfg> {
        // TODO: sequence rules?
        let rule_count = self.inherit.rules().count();
        let history_id =
            self.add_history_node(RootHistoryNode::Origin { origin: rule_count }.into());
        self.inherit.rule(lhs).history(history_id)
    }

    pub fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions<Cfg>> {
        let rule_count = self.inherit.rules().count();
        let history_id =
            self.add_history_node(RootHistoryNode::Origin { origin: rule_count }.into());
        let sequences_to_productions = SequencesToProductions::new(&mut self.inherit);
        SequenceRuleBuilder::new(sequences_to_productions)
            .sequence(lhs)
            .default_history(history_id)
    }

    pub fn binarize(&self) -> BinarizedGrammar {
        BinarizedGrammar {
            inherit: self.inherit.binarize(),
            start: self.start,
            has_wrapped_start: false,
        }
    }
}

impl Deref for Grammar {
    type Target = Cfg;
    fn deref(&self) -> &Self::Target {
        &self.inherit
    }
}

impl DerefMut for Grammar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inherit
    }
}
