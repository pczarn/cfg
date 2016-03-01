//! Precedenced rules are built with the builder pattern.

use std::convert::AsRef;
use std::mem;

use history::AssignPrecedence;
use rule::{GrammarRule, Rule, RuleRef};
use rule::builder::RuleBuilder;
use rule::container::RuleContainer;
use symbol::SymbolSource;

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
const DEFAULT_ASSOC: Associativity = Left;

/// Precedenced rules are built in series of rule alternatives with equal precedence.
pub struct PrecedencedRuleBuilder<D> where
            D: RuleContainer,
            D::History: AssignPrecedence + Default {
    rules: Option<D>,
    lhs: D::Symbol,
    tighter_lhs: D::Symbol,
    current_lhs: D::Symbol,
    history: Option<D::History>,
    assoc: Associativity,
    looseness: u32,
    rules_with_group_assoc: Vec<Rule<D::History, D::Symbol>>,
}

impl<D> PrecedencedRuleBuilder<D>
    where D: RuleContainer,
          D::History: AssignPrecedence + Default
{
    /// Returns a precedenced rule builder.
    pub fn new(mut rules: D, lhs: D::Symbol) -> Self {
        let tightest_lhs = rules.next_sym();
        PrecedencedRuleBuilder {
            rules: Some(rules),
            lhs: lhs,
            tighter_lhs: tightest_lhs,
            current_lhs: tightest_lhs,
            history: None,
            assoc: Left,
            looseness: 0,
            rules_with_group_assoc: vec![],
        }
    }

    /// Starts building a new precedenced rule. The differences in precedence among rules only
    /// matter within a particular precedenced rule.
    pub fn precedenced_rule(mut self, lhs: D::Symbol)
                -> PrecedencedRuleBuilder<D> {
        self.finalize().precedenced_rule(lhs)
    }

    /// Starts building a new grammar rule.
    pub fn rule(mut self, lhs: D::Symbol) -> RuleBuilder<D> {
        self.finalize().rule(lhs)
    }

    /// Assigns the rule history, which is used on the next call to `rhs`, unless overwritten by
    /// a call to `rhs_with_history`.
    pub fn history(mut self, history: D::History) -> Self {
        self.history = Some(history);
        self
    }

    /// Creates a rule alternative. If history wasn't provided, the rule has the `Default` history.
    pub fn rhs<S>(mut self, syms: S) -> Self where
                S: AsRef<[D::Symbol]>,
                D::History: Default {
        let history = self.history.take().unwrap_or_else(||
            D::History::default()
        );
        self.rhs_with_history(syms.as_ref(), history)
    }

    /// Creates a rule alternative with the given RHS and history.
    pub fn rhs_with_history<S>(mut self, syms: S, history: D::History) -> Self
        where S: AsRef<[D::Symbol]>
    {
        let syms = syms.as_ref();
        let this_rule_ref = RuleRef {
            lhs: self.current_lhs,
            rhs: syms,
            history: &(),
        };
        history.assign_precedence(&this_rule_ref, self.looseness);
        let lhs = self.lhs;
        let mut syms = syms.to_vec();
        if self.assoc == Group {
            self.rules_with_group_assoc.push(Rule::new(self.current_lhs, syms, history));
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
            self.rules.as_mut().unwrap().add_rule(self.current_lhs, &syms[..], history);
        }
        // Reset to default associativity and no history.
        self.assoc = DEFAULT_ASSOC;
        self.history = None;
        self
    }

    /// Assigns the associativity, which influences the next call to `rhs` or `rhs_with_history`.
    pub fn associativity(mut self, assoc: Associativity) -> Self
        where D::History: AssignPrecedence,
              D: SymbolSource
    {
        self.assoc = assoc;
        self
    }

    /// Assigns lower precedence to rule alternatives that are built after this call.
    pub fn lower_precedence(mut self) -> Self {
        self.looseness += 1;

        self.tighter_lhs = self.current_lhs;
        self.current_lhs = self.rules.as_mut().unwrap().next_sym();

        RuleBuilder::new(self.rules.as_mut().unwrap()).rule(self.current_lhs)
                                                      .rhs(&[self.tighter_lhs]);
        self
    }

    /// This internal method must be called to finalize the precedenced rule construction.
    fn finalize(&mut self) -> RuleBuilder<D> {
        let mut destination = self.rules.take().unwrap();
        let rules_with_group_assoc = mem::replace(&mut self.rules_with_group_assoc, vec![]);
        let loosest_lhs = self.current_lhs;
        for mut rule in rules_with_group_assoc {
            for sym in &mut rule.rhs {
                if *sym == self.lhs {
                    *sym = loosest_lhs;
                }
            }
            destination.add_rule(rule.lhs(), &rule.rhs[..], rule.history);
        }
        // The associativity is not reset in the call to `rhs`.
        RuleBuilder::new(destination).rule(self.lhs).rhs(&[loosest_lhs])
    }
}

impl<D> Drop for PrecedencedRuleBuilder<D> where
            D: RuleContainer,
            D::History: AssignPrecedence + Default {
    fn drop(&mut self) {
        self.finalize();
    }
}
