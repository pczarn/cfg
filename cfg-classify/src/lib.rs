//! Classification of rules and grammars.

// mod linear;
// mod recursive;
// pub mod cyclical;
// #[cfg(feature = "cfg-predict-sets")]
// pub mod ll;
// pub mod lr;

#[cfg(feature = "cyclical")]
pub use cfg_classify_cyclical::*;
// #[cfg(feature = "linear")]
// pub use cfg_classify_linear::*;
#[cfg(feature = "ll")]
pub use cfg_classify_ll::*;
#[cfg(feature = "lr")]
pub use cfg_classify_lr::*;
#[cfg(feature = "recursive")]
pub use cfg_classify_recursive::*;
#[cfg(feature = "useful")]
pub use cfg_classify_useful::*;
use cfg_grammar::Cfg;
use cfg_symbol::Symbol;

pub trait CfgClassifyExt {
    fn ll_parse_table(&self) -> LlParseTable;
    fn lr0_fsm_builder(&mut self) -> Lr0FsmBuilder;
    fn lr0_closure_builder(&mut self) -> Lr0ClosureBuilder;
    fn recursion(&self) -> Recursion;
    fn make_proper(&mut self) -> bool;
    fn usefulness(&mut self) -> Usefulness;
    fn usefulness_with_roots(&mut self, roots: &[Symbol]) -> Usefulness;
}

impl CfgClassifyExt for Cfg {
    fn ll_parse_table(&self) -> LlParseTable {
        LlParseTable::new(self)
    }

    fn recursion(&self) -> Recursion {
        Recursion::new(self)
    }

    fn lr0_fsm_builder(&mut self) -> Lr0FsmBuilder {
        Lr0FsmBuilder::new(self)
    }

    fn lr0_closure_builder(&mut self) -> Lr0ClosureBuilder {
        Lr0ClosureBuilder::new(self)
    }

    fn usefulness(&mut self) -> Usefulness {
        let mut usefulness = Usefulness::new(self);
        let roots = self.roots();
        usefulness.reachable(roots);
        usefulness
    }

    fn usefulness_with_roots(&mut self, roots: &[Symbol]) -> Usefulness {
        let mut usefulness = Usefulness::new(self);
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
