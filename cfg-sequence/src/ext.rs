use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

use crate::{
    builder::SequenceRuleBuilder, destination::SequenceDestination, rewrite::SequencesToProductions,
};

/// Extension trait for easy adding sequence rules
/// to a `Cfg`.
pub trait CfgSequenceExt {
    /// Adds a sequence rule to a grammar.
    fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions<'_>>;
}

impl CfgSequenceExt for Cfg {
    fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<SequencesToProductions<'_>> {
        SequencesToProductions::new(self).sequence(lhs)
    }
}
