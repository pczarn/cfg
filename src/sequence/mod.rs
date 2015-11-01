//! Sequences are similar to regex repetitions with numbering.

pub mod builder;
pub mod destination;
pub mod rewrite;

use self::Separator::*;

/// Sequence rule representation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Sequence<H, S> {
    /// The rule's left-hand side.
    pub lhs: S,
    /// The rule's right-hand side.
    pub rhs: S,
    /// The minimum number of repetitions.
    pub start: u32,
    /// Either the inclusive maximum number of repetitions, or `None` if the number of repetitions
    /// is unlimited.
    pub end: Option<u32>,
    /// The way elements are separated in a sequence, or `Null`.
    pub separator: Separator<S>,
    /// The history carried with the sequence rule.
    pub history: H,
}

/// The separator symbol and mode of separation in a sequence, or `Null` for no separation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Separator<S> {
    /// Separation with the trailing separator included. In other words, all elements are followed
    /// by the separator.
    Trailing(S),
    /// The separator occurs between elements.
    Proper(S),
    /// The union of `Trailing` and `Proper`. In other words, the trailing separator may or may not
    /// be present.
    Liberal(S),
    /// No separation.
    Null,
}


impl<H, S> Sequence<H, S> {
    /// Assigns the inclusive range of the number of repetitions.
    pub fn inclusive(mut self, start: u32, end: Option<u32>) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    /// Assigns the separator symbol and mode of separation.
    pub fn separator(mut self, sep: Separator<S>) -> Self {
        self.separator = sep;
        self
    }
}

impl<S> Separator<S> {
    /// Returns the kind of separation for a prefix sequence.
    pub fn prefix_separator(self) -> Self {
        match self {
            Proper(sep) | Liberal(sep) => Trailing(sep),
            other => other,
        }
    }
}
