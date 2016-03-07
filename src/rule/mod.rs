//! This module defines grammar rules. Each rule in a context-free grammar
//! consists of a single symbol on its left-hand side and an array of symbols
//! on its right-hand side. In this library, each rule carries additional
//! value called "history."

pub mod builder;
pub mod container;

use symbol::Symbol;

/// Trait for rules of a context-free grammar.
pub trait GrammarRule {
    /// The type of history carried with the rule.
    type History;

    /// Returns the rule's left-hand side.
    fn lhs(&self) -> Symbol;
    /// Returns the rule's right-hand side.
    fn rhs(&self) -> &[Symbol];
    /// Returns a reference to the history carried with the rule.
    fn history(&self) -> &Self::History;
}

impl<'a, R> GrammarRule for &'a R where R: GrammarRule
{
    type History = R::History;

    fn lhs(&self) -> Symbol {
        (**self).lhs()
    }
    fn rhs(&self) -> &[Symbol] {
        (**self).rhs()
    }
    fn history(&self) -> &Self::History {
        (**self).history()
    }
}

/// Typical grammar rule representation.
#[derive(Clone, Debug)]
pub struct Rule<H> {
    lhs: Symbol,
    /// The rule's right-hand side.
    pub rhs: Vec<Symbol>,
    /// The rule's history.
    pub history: H,
}

impl<H> GrammarRule for Rule<H> {
    type History = H;

    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        &self.rhs
    }

    fn history(&self) -> &H {
        &self.history
    }
}

impl<H> Rule<H> {
    /// Creates a new rule.
    pub fn new(lhs: Symbol, rhs: Vec<Symbol>, history: H) -> Self {
        Rule {
            lhs: lhs,
            rhs: rhs,
            history: history,
        }
    }
}

/// References rule's components.
pub struct RuleRef<'a, H: 'a> {
    /// Left-hand side.
    pub lhs: Symbol,
    /// Right-hand side.
    pub rhs: &'a [Symbol],
    /// The rule's history.
    pub history: &'a H,
}

// Can't derive because of the type parameter.
impl<'a, H> Copy for RuleRef<'a, H> {}

// Can't derive because of the where clause.
impl<'a, H> Clone for RuleRef<'a, H> {
    fn clone(&self) -> Self {
        RuleRef {
            lhs: self.lhs,
            rhs: self.rhs,
            history: self.history.clone(),
        }
    }
}

impl<'a, H> GrammarRule for RuleRef<'a, H> {
    type History = H;

    fn lhs(&self) -> Symbol {
        self.lhs
    }

    fn rhs(&self) -> &[Symbol] {
        self.rhs
    }

    fn history(&self) -> &H {
        &self.history
    }
}
