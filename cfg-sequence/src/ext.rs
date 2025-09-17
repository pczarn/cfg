use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

use crate::{
    builder::SequenceRuleBuilder, destination::SequenceDestination, rewrite::SequencesToProductions,
};

pub trait CfgSequenceExt {
    fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions<'_>>;
}

impl CfgSequenceExt for Cfg {
    fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions<'_>> {
        SequencesToProductions::new(self).sequence(lhs)
    }
}
