//! Grammar rules can be built with the builder pattern.

use std::convert::AsRef;

use crate::history::{HistoryNodeRhs, RootHistoryNode};
use crate::precedence::PrecedencedRuleBuilder;
use crate::prelude::*;

/// The rule builder.
pub struct RuleBuilder<C>
where
    C: RuleContainer,
{
    lhs: Option<Symbol>,
    history: Option<HistoryId>,
    rules: C,
}

impl<C> RuleBuilder<C>
where
    C: RuleContainer,
{
    /// Creates a rule builder.
    pub fn new(rules: C) -> RuleBuilder<C> {
        RuleBuilder {
            lhs: None,
            history: None,
            rules: rules,
        }
    }
}

impl<C> RuleBuilder<C>
where
    C: RuleContainer,
{
    /// Starts building a new rule with the given LHS.
    pub fn rule(mut self, lhs: Symbol) -> Self {
        self.lhs = Some(lhs);
        self.history = None;
        self
    }

    /// Assigns the rule history, which is used on the next call to `rhs`, or overwritten by a call
    /// to `rhs_with_history`.
    pub fn history(mut self, history: HistoryId) -> Self {
        self.history = Some(history);
        self
    }

    /// Adds a rule alternative to the grammar. If history wasn't provided, the rule has the
    /// `Default` history.
    pub fn rhs<S>(mut self, syms: S) -> Self
    where
        S: AsRef<[Symbol]>,
    {
        let new_history = match self.history.take() {
            Some(history) => self.rules.add_history_node(
                HistoryNodeRhs {
                    prev: history,
                    rhs: syms.as_ref().to_vec(),
                }
                .into(),
            ),
            None => {
                let base_id = self.rules.add_history_node(
                    RootHistoryNode::Rule {
                        lhs: self.lhs.unwrap(),
                    }
                    .into(),
                );
                self.rules.add_history_node(
                    HistoryNodeRhs {
                        prev: base_id,
                        rhs: syms.as_ref().to_vec(),
                    }
                    .into(),
                )
            }
        };
        self.rhs_with_history(syms, new_history)
    }

    /// Adds a rule alternative with the given RHS and history to the grammar.
    pub fn rhs_with_history<Sr>(mut self, syms: Sr, history_id: HistoryId) -> Self
    where
        Sr: AsRef<[Symbol]>,
    {
        self.rules
            .add_rule(self.lhs.unwrap(), syms.as_ref(), history_id);
        self
    }

    /// Starts building a new precedenced rule.
    pub fn precedenced_rule(self, lhs: Symbol) -> PrecedencedRuleBuilder<C> {
        PrecedencedRuleBuilder::new(self.rules, lhs)
    }
}
