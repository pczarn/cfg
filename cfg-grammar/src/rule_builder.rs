//! Grammar rules can be built with the builder pattern.

use std::convert::AsRef;

use crate::local_prelude::*;
use cfg_history::{LinkedHistoryNode, RootHistoryNode};

/// The rule builder.
pub struct RuleBuilder<'a> {
    lhs: Option<Symbol>,
    history: Option<HistoryId>,
    grammar: &'a mut Cfg,
}

impl<'a> RuleBuilder<'a> {
    /// Creates a rule builder.
    pub fn new(grammar: &'a mut Cfg) -> Self {
        RuleBuilder {
            lhs: None,
            history: None,
            grammar,
        }
    }
}

impl<'a> RuleBuilder<'a> {
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
    pub fn rhs(mut self, syms: impl AsRef<[Symbol]>) -> Self {
        let new_history = match self.history.take() {
            Some(history) => history,
            None => {
                self.grammar.add_history_node(
                    RootHistoryNode::Rule {
                        lhs: self.lhs.unwrap(),
                    }
                    .into(),
                )
            }
        };
        self.rhs_with_history(syms, new_history)
    }

    /// Adds a rule alternative with the given RHS and history to the grammar.
    pub fn rhs_with_history(self, syms: impl AsRef<[Symbol]>, history_id: HistoryId) -> Self {
        let lhs = self.lhs.unwrap();
        let rhs = syms.as_ref().into();
        self.grammar.add_rule(CfgRule {
            lhs,
            rhs,
            history_id,
        });
        self
    }

    /// Adds a rule alternative with the given RHS and history to the grammar.
    pub fn rhs_with_linked_history(
        self,
        syms: impl AsRef<[Symbol]>,
        linked_history: LinkedHistoryNode,
    ) -> Self {
        let history_id = self.grammar.add_multiple_history_nodes(
            RootHistoryNode::Rule {
                lhs: self.lhs.unwrap(),
            },
            [
                linked_history,
            ],
        );
        self.rhs_with_history(syms, history_id)
    }

    /// Starts building a new precedenced rule.
    pub fn precedenced_rule(self, lhs: Symbol) -> PrecedencedRuleBuilder<'a> {
        PrecedencedRuleBuilder::new(self.grammar, lhs)
    }
}
