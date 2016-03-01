//! This module defines grammar rules. Each rule in a context-free grammar
//! consists of a single symbol on its left-hand side and an array of symbols
//! on its right-hand side. In this library, each rule carries additional
//! value called "history."

pub mod builder;
pub mod container;
pub mod terminal_set;

use symbol::GrammarSymbol;

/// Trait for rules of a context-free grammar.
pub trait GrammarRule {
    /// The type of history carried with the rule.
    type History;
    /// The type of symbols.
    type Symbol: GrammarSymbol;

    /// Returns the rule's left-hand side.
    fn lhs(&self) -> Self::Symbol;
    /// Returns the rule's right-hand side.
    fn rhs(&self) -> &[Self::Symbol];
    /// Returns a reference to the history carried with the rule.
    fn history(&self) -> &Self::History;
}

impl<'a, R> GrammarRule for &'a R where R: GrammarRule
{
    type History = R::History;
    type Symbol = R::Symbol;

    fn lhs(&self) -> Self::Symbol {
        (**self).lhs()
    }
    fn rhs(&self) -> &[Self::Symbol] {
        (**self).rhs()
    }
    fn history(&self) -> &Self::History {
        (**self).history()
    }
}

/// Typical grammar rule representation.
#[derive(Clone, Debug)]
pub struct Rule<H, S>
    where S: GrammarSymbol
{
    lhs: S,
    /// The rule's right-hand side.
    pub rhs: Vec<S>,
    /// The rule's history.
    pub history: H,
}

impl<H, S> GrammarRule for Rule<H, S> where S: GrammarSymbol
{
    type Symbol = S;
    type History = H;

    fn lhs(&self) -> S {
        self.lhs
    }

    fn rhs(&self) -> &[S] {
        &self.rhs
    }

    fn history(&self) -> &H {
        &self.history
    }
}

impl<H, S> Rule<H, S> where S: GrammarSymbol
{
    /// Creates a new rule.
    pub fn new(lhs: S, rhs: Vec<S>, history: H) -> Self {
        Rule {
            lhs: lhs,
            rhs: rhs,
            history: history,
        }
    }
}

/// References rule's components.
pub struct RuleRef<'a, H, S>
    where S: GrammarSymbol + 'a,
          H: 'a
{
    /// Left-hand side.
    pub lhs: S,
    /// Right-hand side.
    pub rhs: &'a [S],
    /// The rule's history.
    pub history: &'a H,
}

// Can't derive because of the type parameter.
impl<'a, H, S> Copy for RuleRef<'a, H, S> where S: GrammarSymbol
{}

// Can't derive because of the where clause.
impl<'a, H, S> Clone for RuleRef<'a, H, S> where S: GrammarSymbol
{
    fn clone(&self) -> Self {
        RuleRef {
            lhs: self.lhs,
            rhs: self.rhs,
            history: self.history.clone(),
        }
    }
}

impl<'a, H, S> GrammarRule for RuleRef<'a, H, S> where S: GrammarSymbol
{
    type Symbol = S;
    type History = H;

    fn lhs(&self) -> S {
        self.lhs
    }

    fn rhs(&self) -> &[S] {
        self.rhs
    }

    fn history(&self) -> &H {
        &self.history
    }
}
