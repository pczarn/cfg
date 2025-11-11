//! Classification of rules and grammars.

#![deny(unsafe_code)]
#![deny(missing_docs)]

use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

pub mod cyclical;
// pub mod linear;
#[cfg(feature = "ll")]
pub mod ll;
#[cfg(feature = "lr")]
pub mod lr;
pub mod recursive;
// pub mod regular;
pub mod useful;

/// Extension trait for assigning categories to context-free
/// grammar rules, and related operations.
pub trait CfgClassifyExt {
    /// Computes the LL(1) parse table.
    #[cfg(feature = "ll")]
    fn ll_parse_table(&self) -> ll::LlParseTable<'_>;
    /// Allows you to build the LR(0) fsm.
    #[cfg(feature = "lr")]
    fn lr0_fsm_builder(&mut self) -> lr::Lr0FsmBuilder<'_>;
    /// Allows you to build the LR(0) closure.
    #[cfg(feature = "lr")]
    fn lr0_closure_builder(&mut self) -> lr::Lr0ClosureBuilder<'_>;
    /// Allows you to access information on rule recursiveness.
    fn recursion(&self) -> recursive::Recursion<'_>;
    /// Modifies this grammar in-place to remove useless
    /// rules.
    fn make_proper(&mut self) -> bool;
    /// Determines usefulness of rules with the grammar's roots.
    ///
    /// A rule is useful if it is reachable from the grammar's roots,
    /// and productive.
    fn usefulness(&mut self) -> useful::Usefulness;
    /// Determines usefulness of rules with the given roots.
    ///
    /// A rule is useful if it is reachable from these roots, and
    /// productive.
    fn usefulness_with_roots(&mut self, roots: &[Symbol]) -> useful::Usefulness;
}

impl CfgClassifyExt for Cfg {
    #[cfg(feature = "ll")]
    fn ll_parse_table(&self) -> ll::LlParseTable<'_> {
        ll::LlParseTable::new(self)
    }

    fn recursion(&self) -> recursive::Recursion<'_> {
        recursive::Recursion::new(self)
    }

    #[cfg(feature = "lr")]
    fn lr0_fsm_builder(&mut self) -> lr::Lr0FsmBuilder<'_> {
        lr::Lr0FsmBuilder::new(self)
    }

    #[cfg(feature = "lr")]
    fn lr0_closure_builder(&mut self) -> lr::Lr0ClosureBuilder<'_> {
        lr::Lr0ClosureBuilder::new(self)
    }

    fn usefulness(&mut self) -> useful::Usefulness {
        let mut usefulness = useful::Usefulness::new(self);
        let roots = self.roots();
        usefulness.reachable(roots);
        usefulness
    }

    fn usefulness_with_roots(&mut self, roots: &[Symbol]) -> useful::Usefulness {
        let mut usefulness = useful::Usefulness::new(self);
        usefulness.reachable(roots);
        usefulness
    }

    fn make_proper(&mut self) -> bool {
        let usefulness = self.usefulness();
        let contains_useless_rules = !usefulness.all_useful();
        if contains_useless_rules {
            // for useless in usefulness.useless_rules() {
            //     let rhs: Vec<_> = useless.rule.rhs().iter().map(|t| tok.get(t.usize())).collect();
            //     println!("lhs:{:?} rhs:{:?} unreachable:{} unproductive:{}", tok.get(useless.rule.lhs().usize()), rhs, useless.unreachable, useless.unproductive);
            // }
            // println!("warning: grammar has useless rules");
            usefulness.remove_useless_rules(self);
        }
        !contains_useless_rules
    }
}
