//! This module defines grammar rules. Each rule in a context-free grammar
//! consists of a single symbol on its left-hand side and an array of symbols
//! on its right-hand side. In this library, each rule carries additional
//! value called "history."

pub mod builder;

use crate::prelude::*;

/// Trait for rules of a context-free grammar.
pub trait GrammarRule {
    /// Returns the rule's left-hand side.
    fn lhs(&self) -> Symbol;
    /// Returns the rule's right-hand side.
    fn rhs(&self) -> &[Symbol];
    /// Returns a reference to the history carried with the rule.
    fn history_id(&self) -> HistoryId;

    fn as_ref(&self) -> RuleRef {
        RuleRef {
            lhs: self.lhs(),
            rhs: self.rhs(),
            history_id: self.history_id(),
        }
    }
}

impl<'a, R> GrammarRule for &'a R
where
    R: GrammarRule,
{
    fn lhs(&self) -> Symbol {
        (**self).lhs()
    }
    fn rhs(&self) -> &[Symbol] {
        (**self).rhs()
    }
    fn history_id(&self) -> HistoryId {
        (**self).history_id()
    }
}

/// Typical grammar rule representation.
#[derive(Clone, Debug)]
pub struct Rule {
    lhs: Symbol,
    /// The rule's right-hand side.
    pub rhs: Vec<Symbol>,
    /// The rule's history.
    pub history_id: HistoryId,
}

impl GrammarRule for Rule {
    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        &self.rhs
    }

    fn history_id(&self) -> HistoryId {
        self.history_id
    }
}

impl Rule {
    /// Creates a new rule.
    pub fn new(lhs: Symbol, rhs: Vec<Symbol>, history_id: HistoryId) -> Self {
        Rule {
            lhs: lhs,
            rhs: rhs,
            history_id: history_id,
        }
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

impl<'a> GrammarRule for RuleRef<'a> {
    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        self.rhs
    }

    fn history_id(&self) -> HistoryId {
        self.history_id
    }
}
