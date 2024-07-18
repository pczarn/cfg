//! Precedenced rules are built with the builder pattern.

use std::convert::AsRef;

use crate::cfg::CfgRule;
use crate::history::{HistoryNodeAssignPrecedence, HistoryNodeRhs, RootHistoryNode};
use crate::local_prelude::*;
use crate::rule_builder::RuleBuilder;

use self::Associativity::*;

/// Specifies the associativity of an operator.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Associativity {
    /// Left associative.
    Left,
    /// Right associative.
    Right,
    /// `Group` usually means the operand is delimited, e.g. by parentheses.
    Group,
}

/// The default associativity.
pub const DEFAULT_ASSOC: Associativity = Left;

/// Precedenced rules are built in series of rule alternatives with equal precedence.
pub struct PrecedencedRuleBuilder<D>
where
    D: RuleContainer,
{
    rules: D,
    lhs: Symbol,
    tighter_lhs: Symbol,
    current_lhs: Symbol,
    history: Option<HistoryId>,
    assoc: Associativity,
    looseness: u32,
    rules_with_group_assoc: Vec<CfgRule>,
}

impl<D> PrecedencedRuleBuilder<D>
where
    D: RuleContainer,
{
    /// Returns a precedenced rule builder.
    pub fn new(mut rules: D, lhs: Symbol) -> Self {
        let tightest_lhs = rules.next_sym();
        PrecedencedRuleBuilder {
            rules,
            lhs,
            tighter_lhs: tightest_lhs,
            current_lhs: tightest_lhs,
            history: None,
            assoc: Left,
            looseness: 0,
            rules_with_group_assoc: vec![],
        }
    }
}

impl<D> PrecedencedRuleBuilder<D>
where
    D: RuleContainer,
{
    /// Starts building a new precedenced rule. The differences in precedence among rules only
    /// matter within a particular precedenced rule.
    pub fn precedenced_rule(self, lhs: Symbol) -> PrecedencedRuleBuilder<D> {
        self.finalize().precedenced_rule(lhs)
    }

    /// Starts building a new grammar rule.
    pub fn rule(self, lhs: Symbol) -> RuleBuilder<D> {
        self.finalize().rule(lhs)
    }

    /// Assigns the rule history, which is used on the next call to `rhs`, unless overwritten by
    /// a call to `rhs_with_history`.
    #[must_use]
    pub fn history(mut self, history: HistoryId) -> Self {
        self.history = Some(history);
        self
    }

    /// Creates a rule alternative. If history wasn't provided, the rule has the `Default` history.
    #[must_use]
    pub fn rhs<S>(mut self, syms: S) -> Self
    where
        S: AsRef<[Symbol]>,
    {
        let history_id = self.history.take().unwrap_or_else(|| {
            self.rules
                .add_history_node(RootHistoryNode::Rule { lhs: self.lhs }.into())
        });
        let history_id = self.rules.add_history_node(
            HistoryNodeRhs {
                prev: history_id,
                rhs: syms.as_ref().to_vec(),
            }
            .into(),
        );
        self.rhs_with_history(syms.as_ref(), history_id)
    }

    /// Creates a rule alternative with the given RHS and history.
    #[must_use]
    pub fn rhs_with_history<S>(mut self, syms: S, history_id: HistoryId) -> Self
    where
        S: AsRef<[Symbol]>,
    {
        let syms = syms.as_ref();
        let history_assign_precedence = self.rules.add_history_node(
            HistoryNodeAssignPrecedence {
                prev: history_id,
                looseness: self.looseness,
            }
            .into(),
        );
        let lhs = self.lhs;
        let mut syms = syms.to_vec();
        if self.assoc == Group {
            self.rules_with_group_assoc.push(CfgRule::new(
                self.current_lhs,
                syms,
                history_assign_precedence,
            ));
        } else {
            {
                // Symbols equal to the LHS symbol.
                let mut iter = syms.iter_mut().filter(|&&mut sym| sym == lhs);
                let extreme_sym_mut = if self.assoc == Left {
                    // Leftmost one.
                    iter.next()
                } else {
                    // Rightmost one.
                    iter.next_back()
                };
                if let Some(extreme_sym) = extreme_sym_mut {
                    *extreme_sym = self.current_lhs;
                }
                for sym in iter {
                    *sym = self.tighter_lhs;
                }
            };
            self.rules.add_rule(RuleRef {
                lhs: self.current_lhs,
                rhs: &syms[..],
                history_id: history_assign_precedence,
            });
        }
        // Reset to default associativity and no history.
        self.assoc = DEFAULT_ASSOC;
        self.history = None;
        self
    }

    /// Assigns the associativity, which influences the next call to `rhs` or `rhs_with_history`.
    #[must_use]
    pub fn associativity(mut self, assoc: Associativity) -> Self {
        self.assoc = assoc;
        self
    }

    /// Assigns lower precedence to rule alternatives that are built after this call.
    #[must_use]
    pub fn lower_precedence(mut self) -> Self {
        self.looseness += 1;

        self.tighter_lhs = self.current_lhs;
        self.current_lhs = self.rules.next_sym();

        let history_id = self.rules.add_history_node(RootHistoryNode::NoOp.into());
        RuleBuilder::new(&mut self.rules)
            .rule(self.current_lhs)
            .rhs_with_history(&[self.tighter_lhs], history_id);
        self
    }

    /// This internal method must be called to finalize the precedenced rule construction.
    pub fn finalize(mut self) -> RuleBuilder<D> {
        let loosest_lhs = self.current_lhs;
        for mut rule in self.rules_with_group_assoc.drain(..) {
            for sym in &mut rule.rhs {
                if *sym == self.lhs {
                    *sym = loosest_lhs;
                }
            }
            self.rules.add_rule(rule.as_rule_ref());
        }
        let history_id = self.rules.add_history_node(RootHistoryNode::NoOp.into());
        // The associativity is not reset in the call to `rhs`.
        RuleBuilder::new(self.rules)
            .rule(self.lhs)
            .rhs_with_history(&[loosest_lhs], history_id)
    }
}
