//! Sequences are similar to regex repetitions with numbering.

pub mod builder;
pub mod destination;
pub mod rewrite;
mod ext;

pub use crate::ext::CfgSequenceExt;

use std::ops::{Bound, RangeBounds};

use cfg_history::HistoryId;
use cfg_symbol::Symbol;

use self::Separator::*;

/// Sequence rule representation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Sequence {
    /// The rule's left-hand side.
    pub lhs: Symbol,
    /// The rule's right-hand side.
    pub rhs: Symbol,
    /// The minimum number of repetitions.
    pub start: u32,
    /// Either the inclusive maximum number of repetitions, or `None` if the number of repetitions
    /// is unlimited.
    pub end: Option<u32>,
    /// The way elements are separated in a sequence, or `Null`.
    pub separator: Separator,
    /// The history carried with the sequence rule.
    pub history_id: Option<HistoryId>,
}

/// The separator symbol and mode of separation in a sequence, or `Null` for no separation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Separator {
    /// Separation with the trailing separator included. In other words, all elements are followed
    /// by the separator.
    Trailing(Symbol),
    /// The separator occurs between elements.
    Proper(Symbol),
    /// The union of `Trailing` and `Proper`. In other words, the trailing separator may or may not
    /// be present.
    Liberal(Symbol),
    /// No separation.
    Null,
}

impl Sequence {
    /// Assigns the inclusive range of the number of repetitions.
    pub fn inclusive(mut self, start: u32, end: Option<u32>) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    /// Assigns the separator symbol and mode of separation.
    pub fn separator(mut self, sep: Separator) -> Self {
        self.separator = sep;
        self
    }

    /// Adds a range to the sequence.
    pub fn range(self, range: impl RangeBounds<u32>) -> Self {
        let to_option = |bound: Bound<u32>, diff| match bound {
            Bound::Included(included) => Some(included),
            Bound::Excluded(excluded) => Some((excluded as i64 + diff) as u32),
            Bound::Unbounded => None,
        };
        self.inclusive(
            to_option(range.start_bound().cloned(), 1).unwrap_or(0),
            to_option(range.end_bound().cloned(), -1),
        )
    }
}

impl Separator {
    /// Returns the kind of separation for a prefix sequence.
    pub fn prefix_separator(self) -> Self {
        match self {
            Proper(sep) | Liberal(sep) => Trailing(sep),
            other => other,
        }
    }
}

impl From<Separator> for Option<Symbol> {
    fn from(val: Separator) -> Option<Symbol> {
        match val {
            Trailing(sep) => Some(sep),
            _ => None,
        }
    }
}
