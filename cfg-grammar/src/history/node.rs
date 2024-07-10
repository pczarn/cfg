//! Any data carried alongside a grammar rule can be its _history_. Rule histories may contain
//! more than semantic actions.

use std::num::NonZeroUsize;

use crate::Symbol;

pub type HistoryId = NonZeroUsize;

#[derive(Clone)]
pub enum HistoryNode {
    Linked {
        prev: HistoryId,
        node: LinkedHistoryNode,
    },
    Root(RootHistoryNode),
}

#[derive(Clone)]
pub enum LinkedHistoryNode {
    Rhs {
        rhs: Vec<Symbol>,
    },
    Binarize {
        depth: u32,
    },
    EliminateNulling {
        rhs0: Symbol,
        rhs1: Option<Symbol>,
        which: BinarizedRhsSubset,
    },
    AssignPrecedence {
        looseness: u32,
    },
    RewriteSequence {
        top: bool,
        rhs: Symbol,
        sep: Option<Symbol>,
    },
    Weight {
        weight: f64,
    },
    Distances {
        events: Vec<u32>,
    },
}

#[derive(Clone, Copy)]
pub enum RootHistoryNode {
    NoOp,
    Rule { lhs: Symbol },
    Origin { origin: usize },
}

impl From<RootHistoryNode> for HistoryNode {
    fn from(value: RootHistoryNode) -> Self {
        HistoryNode::Root(value)
    }
}

pub struct HistoryNodeRhs {
    pub prev: HistoryId,
    pub rhs: Vec<Symbol>,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeBinarize {
    pub prev: HistoryId,
    pub depth: u32,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeWeight {
    pub prev: HistoryId,
    pub weight: f64,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeEliminateNulling {
    pub prev: HistoryId,
    pub rhs0: Symbol,
    pub rhs1: Option<Symbol>,
    pub which: BinarizedRhsSubset,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeAssignPrecedence {
    pub prev: HistoryId,
    pub looseness: u32,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeRewriteSequence {
    pub prev: HistoryId,
    pub top: bool,
    pub rhs: Symbol,
    pub sep: Option<Symbol>,
}

impl From<HistoryNodeRhs> for HistoryNode {
    fn from(value: HistoryNodeRhs) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::Rhs { rhs: value.rhs },
        }
    }
}

impl From<HistoryNodeBinarize> for HistoryNode {
    fn from(value: HistoryNodeBinarize) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::Binarize { depth: value.depth },
        }
    }
}

impl From<HistoryNodeWeight> for HistoryNode {
    fn from(value: HistoryNodeWeight) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::Weight {
                weight: value.weight,
            },
        }
    }
}

impl From<HistoryNodeEliminateNulling> for HistoryNode {
    fn from(value: HistoryNodeEliminateNulling) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::EliminateNulling {
                rhs0: value.rhs0,
                rhs1: value.rhs1,
                which: value.which,
            },
        }
    }
}

impl From<HistoryNodeAssignPrecedence> for HistoryNode {
    fn from(value: HistoryNodeAssignPrecedence) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::AssignPrecedence {
                looseness: value.looseness,
            },
        }
    }
}

impl From<HistoryNodeRewriteSequence> for HistoryNode {
    fn from(value: HistoryNodeRewriteSequence) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::RewriteSequence {
                top: value.top,
                rhs: value.rhs,
                sep: value.sep,
            },
        }
    }
}

/// Used to inform which symbols on a rule'Symbol RHS are nullable, and will be eliminated.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BinarizedRhsSubset {
    /// The first of two symbols.
    Left,
    /// The second of two symbols.
    Right,
    /// All 1 or 2 symbols. The rule is nullable.
    All,
}
