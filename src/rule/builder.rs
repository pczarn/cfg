//! Grammar rules can be built with the builder pattern.

use std::convert::AsRef;

use history::{AssignPrecedence, DefaultHistory, HistoryFn};
use precedence::PrecedencedRuleBuilder;
use rule::container::RuleContainer;
use symbol::SymbolSource;

/// The rule builder.
pub struct RuleBuilder<C, F = DefaultHistory<<C as RuleContainer>::History,
                                             <C as SymbolSource>::Symbol>>
        where C: RuleContainer {
    lhs: Option<C::Symbol>,
    history: Option<C::History>,
    default_history: F,
    rules: C,
}

impl<C> RuleBuilder<C> where C: RuleContainer
{
    /// Creates a rule builder.
    pub fn new(rules: C) -> RuleBuilder<C> {
        RuleBuilder {
            lhs: None,
            history: None,
            default_history: DefaultHistory::new(),
            rules: rules,
        }
    }
}

impl<C, F> RuleBuilder<C, F> where C: RuleContainer {
    /// Sets the default history source.
    pub fn default_history<F2>(self, f: F2) -> RuleBuilder<C, F2> {
        RuleBuilder {
            lhs: self.lhs,
            history: self.history,
            default_history: f,
            rules: self.rules,
        }
    }

    /// Starts building a new rule with the given LHS.
    pub fn rule(mut self, lhs: C::Symbol) -> Self {
        self.lhs = Some(lhs);
        self.history = None;
        self
    }

    /// Assigns the rule history, which is used on the next call to `rhs`, or overwritten by a call
    /// to`rhs_with_history`.
    pub fn history(mut self, history: C::History) -> Self {
        self.history = Some(history);
        self
    }

    /// Adds a rule alternative to the grammar. If history wasn't provided, the rule has the
    /// `Default` history.
    pub fn rhs<Sr>(mut self, syms: Sr) -> Self where
                Sr: AsRef<[C::Symbol]>,
                F: HistoryFn<C::History, C::Symbol> {
        let history = self.history.take().unwrap_or_else(||
            self.default_history.call_mut(self.lhs.unwrap(), syms.as_ref())
        );
        self.rhs_with_history(syms, history)
    }

    /// Adds a rule alternative with the given RHS and history to the grammar.
    pub fn rhs_with_history<Sr>(mut self, syms: Sr, history: C::History) -> Self
        where Sr: AsRef<[C::Symbol]>,
    {
        self.rules.add_rule(self.lhs.unwrap(), syms.as_ref(), history);
        self
    }

    /// Starts building a new precedenced rule.
    pub fn precedenced_rule(self, lhs: C::Symbol) -> PrecedencedRuleBuilder<C>
        where C::History: AssignPrecedence + Default
    {
        PrecedencedRuleBuilder::new(self.rules, lhs)
    }
}
