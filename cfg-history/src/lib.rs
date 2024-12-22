//! Any data carried alongside a grammar rule can be its _history_. Rule histories may contain
//! more than semantic actions.

use std::{num::NonZeroUsize, ops};

use cfg_symbol::Symbol;

use self::BinarizedRhsRange::*;

pub type HistoryId = NonZeroUsize;

#[derive(Clone, Debug)]
pub struct HistoryGraph {
    nodes: Vec<HistoryNode>,
}

impl Default for HistoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryGraph {
    pub fn new() -> Self {
        Self {
            nodes: vec![RootHistoryNode::NoOp.into()],
        }
    }

    pub fn next_id(&mut self) -> HistoryId {
        self.nodes
            .len()
            .try_into()
            .expect("problem with zero length history graph")
    }

    pub fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
        let result = self.next_id();
        self.push(node);
        result
    }
}

impl ::std::ops::Deref for HistoryGraph {
    type Target = Vec<HistoryNode>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl ::std::ops::DerefMut for HistoryGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}

#[derive(Clone, Debug)]
pub enum HistoryNode {
    Linked {
        prev: HistoryId,
        node: LinkedHistoryNode,
    },
    Root(RootHistoryNode),
}

#[derive(Clone, Debug)]
pub enum LinkedHistoryNode {
    Binarize {
        height: u32,
        full_len: usize,
        is_top: bool,
    },
    EliminateNulling {
        rhs0: Option<Symbol>,
        rhs1: Option<Symbol>,
        which: BinarizedRhsRange,
    },
    AssignPrecedence {
        looseness: u32,
    },
    RewriteSequence {
        top: bool,
        rhs: Symbol,
        sep: Option<Symbol>,
    },
    SequenceRhs {
        rhs: [Option<Symbol>; 3],
    },
    Weight {
        weight: f64,
    },
    // Distances {
    //     events: Vec<u32>,
    // },
}

#[derive(Clone, Copy, Debug)]
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
    pub height: u32,
    pub full_len: usize,
    pub is_top: bool,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeWeight {
    pub prev: HistoryId,
    pub weight: f64,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeEliminateNulling {
    pub prev: HistoryId,
    pub rhs0: Option<Symbol>,
    pub rhs1: Option<Symbol>,
    pub which: BinarizedRhsRange,
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

#[derive(Clone, Copy)]
pub struct HistoryNodeSequenceRhs {
    pub prev: HistoryId,
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

impl From<HistoryNodeBinarize> for HistoryNode {
    fn from(value: HistoryNodeBinarize) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::Binarize {
                height: value.height,
                full_len: value.full_len,
                is_top: value.is_top,
            },
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

impl From<HistoryNodeSequenceRhs> for HistoryNode {
    fn from(value: HistoryNodeSequenceRhs) -> Self {
        HistoryNode::Linked {
            prev: value.prev,
            node: LinkedHistoryNode::SequenceRhs {
                rhs: value.rhs,
            },
        }
    }
}

/// Used to inform which symbols on a rule'Symbol RHS are nullable, and will be eliminated.
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
    pub fn as_range(self) -> ops::Range<usize> {
        match self {
            Left => 0..1,
            Right => 1..2,
            All(num) => 0..num,
        }
    }
}
