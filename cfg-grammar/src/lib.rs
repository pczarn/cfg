//! Library for manipulations on context-free grammars. Most transformations are abstracted over
//! grammar representations.

#![recursion_limit = "512"]
#![deny(unsafe_code)]
#![deny(
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(test, allow(missing_docs))]

pub mod binarized_cfg;
pub mod cfg;
pub mod history;
pub mod precedenced_rule;
pub mod rhs_closure;
pub mod rule;
pub mod rule_container;
pub mod symbol;

pub use crate::binarized_cfg::BinarizedCfg;
pub use crate::cfg::Cfg;
pub use crate::history::node::{HistoryId, HistoryNode};
pub use crate::rule::AsRuleRef;
pub use crate::rule_container::RuleContainer;
pub use cfg_symbol::source::SymbolSource;
pub use cfg_symbol::Symbol;

pub(crate) mod local_prelude {
    pub use crate::history::node::{HistoryId, HistoryNode};
    pub use crate::rule_container::RuleContainer;
    // pub use crate::rule::AsRuleRef;
    pub use cfg_symbol::source::SymbolSource;
    pub use cfg_symbol::Symbol;
}
