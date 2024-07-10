use crate::local_prelude::*;

use super::{AsRuleRef, RuleRef};

/// Typical grammar rule representation.
#[derive(Clone, Debug)]
pub struct CfgRule {
    pub lhs: Symbol,
    /// The rule's right-hand side.
    pub rhs: Vec<Symbol>,
    /// The rule's history.
    pub history_id: HistoryId,
}

impl CfgRule {
    /// Creates a new rule.
    pub fn new(lhs: Symbol, rhs: Vec<Symbol>, history_id: HistoryId) -> Self {
        CfgRule {
            lhs: lhs,
            rhs: rhs,
            history_id: history_id,
        }
    }
}

impl AsRuleRef for CfgRule {
    // TODO: better fn name?
    fn as_rule_ref(&self) -> RuleRef {
        RuleRef {
            lhs: self.lhs,
            rhs: &self.rhs[..],
            history_id: self.history_id,
        }
    }
}
