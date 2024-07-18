//! Library for manipulations on context-free grammars. Most transformations are abstracted over
//! grammar representations.

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

pub mod cfg;
pub mod precedenced_rule;
pub mod remap_symbols;
pub mod rhs_closure;
pub mod rule_builder;
pub mod rule_ref;
pub mod symbol_set;

pub use crate::cfg::Cfg;
pub use crate::rule_ref::{AsRuleRef, RuleRef};
pub use cfg_symbol::source::SymbolSource;
pub use cfg_symbol::Symbol;

pub(crate) mod local_prelude {
    pub use cfg_history::HistoryId;
    // pub use crate::rule_builder::RuleBuilder;
    pub use crate::rule_ref::{AsRuleRef, RuleRef};
    // pub use crate::symbol_set::SymbolBitSet;
    pub use cfg_symbol::source::SymbolSource;
    pub use cfg_symbol::Symbol;
}
