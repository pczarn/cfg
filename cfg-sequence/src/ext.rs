use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

use crate::{builder::SequenceRuleBuilder, destination::SequenceDestination, rewrite::SequencesToProductions};

pub trait CfgSequenceExt {
    fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions>;
}

impl CfgSequenceExt for Cfg {
    fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions> {
        SequencesToProductions::new(self).sequence(lhs)
    }
}
