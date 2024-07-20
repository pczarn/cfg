//! This module defines grammar rules. Each rule in a context-free grammar
//! consists of a single symbol on its left-hand side and an array of symbols
//! on its right-hand side. In this library, each rule carries additional
//! value called "history."

use smallvec::SmallVec;

use crate::{cfg::CfgRule, local_prelude::*};

/// References rule's components.
#[derive(Copy, Clone)]
pub struct RuleRef<'a> {
    /// Left-hand side.
    pub lhs: Symbol,
    /// Right-hand side.
    pub rhs: &'a [Symbol],
    /// The rule's history.
    pub history_id: HistoryId,
}

impl<'a> From<&'a CfgRule> for RuleRef<'a> {
    fn from(value: &'a CfgRule) -> Self {
        RuleRef {
            lhs: value.lhs,
            rhs: &value.rhs[..],
            history_id: value.history_id,
        }
    }
}

impl<'a> From<RuleRef<'a>> for CfgRule {
    fn from(value: RuleRef<'a>) -> Self {
        CfgRule {
            lhs: value.lhs,
            #[cfg(not(feature = "smallvec"))]
            rhs: value.rhs.to_vec(),
            #[cfg(feature = "smallvec")]
            rhs: SmallVec::from_slice(value.rhs),
            history_id: value.history_id,
        }
    }
}
