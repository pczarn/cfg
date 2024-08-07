//! This module defines grammar rules. Each rule in a context-free grammar
//! consists of a single symbol on its left-hand side and an array of symbols
//! on its right-hand side. In this library, each rule carries additional
//! value called "history."

pub mod builder;
pub mod cfg_rule;

use crate::local_prelude::*;

/// Trait for rules of a context-free grammar.
pub trait AsRuleRef {
    fn as_rule_ref(&self) -> RuleRef;
}

impl<'a, R> AsRuleRef for &'a R
where
    R: AsRuleRef,
{
    fn as_rule_ref(&self) -> RuleRef {
        (**self).as_rule_ref()
    }
}

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

impl<'a> AsRuleRef for RuleRef<'a> {
    fn as_rule_ref(&self) -> RuleRef {
        *self
    }
}
