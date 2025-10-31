//! Any data carried alongside a grammar rule can be its _history_. Rule histories may contain
//! more than semantic actions.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::ops;

use cfg_symbol::Symbol;
use earley::{History, process_linked};

use self::BinarizedRhsRange::*;

pub mod earley;

/// A history update to an existing history value,
/// used with grammar transformations.
#[derive(Clone, Debug)]
pub enum LinkedHistoryNode {
    /// Updates history during binarization of a rule.
    /// Binarization is the process of making a rule with
    /// any number of RHS symbols become a rule with
    /// only one or two symbols on the RHS.
    Binarize {
        /// Tells us how high we are in a chain of binarized rules.
        /// For example, when binarizing a rule A ::= B C D E, we get:
        ///
        /// - g0 ::= B C -- this is at height 0
        ///
        /// - g1 ::= g0 D -- this is at height 1
        ///
        /// - A ::= g1 E -- this is at height 2
        height: u32,
        /// Original length of the rule before binarization.
        full_len: usize,
        /// Whether we are at the highest rule.
        is_top: bool,
    },
    /// Updates history after nulling rule elimination.
    EliminateNulling {
        /// The symbol at the first place in the RHS of the binarized rule
        /// we are de-nulling, or `None` if the rule has zero symbols
        /// to begin with.
        rhs0: Option<Symbol>,
        /// The symbol at the second place in the RHS of the binarized rule
        /// we are de-nulling, or `None` if the rule has zero or one symbol
        /// to begin with.
        rhs1: Option<Symbol>,
        /// Range of eliminated RHS symbols of the binary rule.
        which: BinarizedRhsRange,
    },
    /// Updates history for rules built with the precedenced rule
    /// builder.
    AssignPrecedence {
        /// Operator precedence, starting with highest
        /// and loosening with increase.
        looseness: u32,
    },
    /// Updates history of a rule made from a sequence rule.
    RewriteSequence {
        /// Whether this is the outermost level of derivation
        /// among rules build by the sequence rewrite.
        top: bool,
        /// The symbol we are repeating.
        rhs: Symbol,
        /// Separator.
        sep: Option<Symbol>,
    },
    /// The RHS as computed by the sequence rewrite.
    /// The rule with this history was made from a sequence
    /// rule.
    SequenceRhs {
        /// The RHS as computed by the sequence rewrite.
        /// Up to three symbols may appear on the RHS.
        rhs: [Option<Symbol>; 3],
    },
    /// Weight for generation for PCFGs.
    Weight {
        /// Floating point weight.
        weight: f64,
    },
    // Distances {
    //     events: Vec<u32>,
    // },
}

/// The basic semantics. They may receive further updates.
#[derive(Clone, Copy, Debug)]
pub enum RootHistoryNode {
    /// Empty semantics.
    NoOp,
    /// This is a plain rule.
    Rule {
        /// Only the LHS is given. The RHS may be updated
        /// through [`HistoryNodeRhs`].
        lhs: Symbol,
    },
    /// This is a plain rule.
    Origin {
        /// Original rule ID is given.
        origin: usize,
    },
}

/// Provides information about the RHS of the rule
/// we have semantics for.
pub struct HistoryNodeRhs {
    /// The history we are updating.
    pub prev: History,
    /// The rule RHS.
    pub rhs: Vec<Symbol>,
}

/// Updates history during binarization of a rule.
/// Binarization is the process of making a rule with
/// any number of RHS symbols become a rule with
/// only one or two symbols on the RHS.
#[derive(Clone, Copy)]
pub struct HistoryNodeBinarize {
    /// The history we are updating.
    pub prev: History,
    /// Tells us how high we are in a chain of binarized rules.
    /// For example, when binarizing a rule A ::= B C D E, we get:
    ///
    /// - g0 ::= B C -- this is at height 0
    ///
    /// - g1 ::= g0 D -- this is at height 1
    ///
    /// - A ::= g1 E -- this is at height 2
    pub height: u32,
    /// Original length of the rule before binarization.
    pub full_len: usize,
    /// Whether we are at the highest rule.
    pub is_top: bool,
}

///
#[derive(Clone, Copy)]
pub struct HistoryNodeWeight {
    /// The history we are updating.
    pub prev: History,
    /// Floating-point weight for generation for PCFGs.
    pub weight: f64,
}

/// Updates history after nulling rule elimination.
#[derive(Clone, Copy)]
pub struct HistoryNodeEliminateNulling {
    /// The history we are updating.
    pub prev: History,
    /// The symbol at the first place in the RHS of the binarized rule
    /// we are de-nulling, or `None` if the rule has zero symbols
    /// to begin with.
    pub rhs0: Option<Symbol>,
    /// The symbol at the secong place in the RHS of the binarized rule
    /// we are de-nulling, or `None` if the rule has zero or one symbol
    /// to begin with.
    pub rhs1: Option<Symbol>,
    /// Range of eliminated RHS symbols of the binary rule.
    pub which: BinarizedRhsRange,
}

/// This is built with the precedenced rule builder.
#[derive(Clone, Copy)]
pub struct HistoryNodeAssignPrecedence {
    /// The history we are updating.
    pub prev: History,
    /// Operator precedence, starting with highest
    /// and loosening with increase.
    pub looseness: u32,
}

/// This is built with the sequence rewrite.
#[derive(Clone, Copy)]
pub struct HistoryNodeRewriteSequence {
    /// The history we are updating.
    pub prev: History,
    /// Whether this is the outermost level of derivation.
    pub top: bool,
    /// Symbol we are repating.
    pub rhs: Symbol,
    /// Our separator, or `None` for no separation.
    pub sep: Option<Symbol>,
}

/// The RHS as computed by the sequence rewrite.
#[derive(Clone, Copy)]
pub struct HistoryNodeSequenceRhs {
    /// The history we are updating.
    pub prev: History,
    /// The RHS as computed by the sequence rewrite.
    /// Up to three symbols may appear on the RHS.
    pub rhs: [Option<Symbol>; 3],
}

// impl From<HistoryNodeRhs> for HistoryNode {
//     fn from(value: HistoryNodeRhs) -> Self {
//         HistoryNode::Linked {
//             prev: value.prev,
//             node: LinkedHistoryNode::Rhs { rhs: value.rhs },
//         }
//     }
// }

impl From<HistoryNodeBinarize> for History {
    fn from(value: HistoryNodeBinarize) -> Self {
        process_linked(
            &LinkedHistoryNode::Binarize {
                height: value.height,
                full_len: value.full_len,
                is_top: value.is_top,
            },
            value.prev,
        )
    }
}

impl From<HistoryNodeWeight> for History {
    fn from(value: HistoryNodeWeight) -> Self {
        process_linked(
            &LinkedHistoryNode::Weight {
                weight: value.weight,
            },
            value.prev,
        )
    }
}

impl From<HistoryNodeEliminateNulling> for History {
    fn from(value: HistoryNodeEliminateNulling) -> Self {
        process_linked(
            &LinkedHistoryNode::EliminateNulling {
                rhs0: value.rhs0,
                rhs1: value.rhs1,
                which: value.which,
            },
            value.prev,
        )
    }
}

impl From<HistoryNodeAssignPrecedence> for History {
    fn from(value: HistoryNodeAssignPrecedence) -> Self {
        process_linked(
            &LinkedHistoryNode::AssignPrecedence {
                looseness: value.looseness,
            },
            value.prev,
        )
    }
}

impl From<HistoryNodeRewriteSequence> for History {
    fn from(value: HistoryNodeRewriteSequence) -> Self {
        process_linked(
            &LinkedHistoryNode::RewriteSequence {
                top: value.top,
                rhs: value.rhs,
                sep: value.sep,
            },
            value.prev,
        )
    }
}

impl From<HistoryNodeSequenceRhs> for History {
    fn from(value: HistoryNodeSequenceRhs) -> Self {
        process_linked(
            &LinkedHistoryNode::SequenceRhs { rhs: value.rhs },
            value.prev,
        )
    }
}

/// Used to inform history about which symbols on a rule's
/// Symbol RHS are nullable, and will be eliminated.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BinarizedRhsRange {
    /// The first of two symbols.
    Left,
    /// The second of two symbols.
    Right,
    /// All 0, 1 or 2 symbols. The rule is nullable.
    All(usize),
}

impl BinarizedRhsRange {
    /// Turns this into a Rust `Range` of dot positions in a rule.
    pub fn as_range(self) -> ops::Range<usize> {
        match self {
            Left => 0..1,
            Right => 1..2,
            All(num) => 0..num,
        }
    }

    /// Flips the range to cover the rest of the rule RHS.
    ///
    /// # Panics
    ///
    /// Does not work with `All`.
    pub fn negate(self) -> Self {
        match self {
            Left => Right,
            Right => Left,
            All(_) => unreachable!(),
        }
    }
}
