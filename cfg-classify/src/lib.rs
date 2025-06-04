//! Classification of rules and grammars.

use cfg_symbol::Symbol;
use cfg_grammar::Cfg;

pub mod cyclical;
pub mod linear;
#[cfg(feature = "ll")]
pub mod ll;
#[cfg(feature = "lr")]
pub mod lr;
pub mod recursive;
pub mod regular;
pub mod useful;

pub trait CfgClassifyExt {
#[cfg(feature = "ll")]
    fn ll_parse_table(&self) -> ll::LlParseTable;
#[cfg(feature = "lr")]
    fn lr0_fsm_builder(&mut self) -> lr::Lr0FsmBuilder;
#[cfg(feature = "lr")]
    fn lr0_closure_builder(&mut self) -> lr::Lr0ClosureBuilder;
    fn recursion(&self) -> recursive::Recursion;
    fn make_proper(&mut self) -> bool;
    fn usefulness(&mut self) -> useful::Usefulness;
    fn usefulness_with_roots(&mut self, roots: &[Symbol]) -> useful::Usefulness;
}

impl CfgClassifyExt for Cfg {
#[cfg(feature = "ll")]
    fn ll_parse_table(&self) -> ll::LlParseTable {
        ll::LlParseTable::new(self)
    }

    fn recursion(&self) -> recursive::Recursion {
        recursive::Recursion::new(self)
    }

#[cfg(feature = "lr")]
    fn lr0_fsm_builder(&mut self) -> lr::Lr0FsmBuilder {
        lr::Lr0FsmBuilder::new(self)
    }

#[cfg(feature = "lr")]
    fn lr0_closure_builder(&mut self) -> lr::Lr0ClosureBuilder {
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
