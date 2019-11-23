use std::ops::{Deref, DerefMut};

use Cfg;
use Symbol;
use ContextFree;
use ContextFreeRef;
use rule::builder::RuleBuilder;
use sequence::builder::SequenceRuleBuilder;
use sequence::Sequence;

use super::BinarizedGrammar;
use super::history::{History, BuildHistory};

/// Drop-in replacement for `cfg::Cfg` that traces relations between user-provided
/// and internal grammars.
#[derive(Default)]
pub struct Grammar {
    inherit: Cfg<History, History>,
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

    pub fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut Cfg<History, History>, BuildHistory> {
        let rule_count = self.inherit.rules().count() + self.sequence_rules().len();
        self.inherit.rule(lhs).default_history(BuildHistory::new(rule_count))
    }

    pub fn sequence(&mut self, lhs: Symbol)
        -> SequenceRuleBuilder<History, &mut Vec<Sequence<History>>, BuildHistory>
    {
        let rule_count = self.inherit.rules().count() + self.sequence_rules().len();
        self.inherit.sequence(lhs).default_history(BuildHistory::new(rule_count))
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
    type Target = Cfg<History, History>;
    fn deref(&self) -> &Self::Target {
        &self.inherit
    }
}

impl DerefMut for Grammar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inherit
    }
}
