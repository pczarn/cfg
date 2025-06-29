//! Any data carried alongside a grammar rule can be its _history_. Rule histories may contain
//! more than semantic actions.

use std::{num::NonZeroUsize, ops};

use cfg_symbol::Symbol;
use earley::{process_linked, History};

use self::BinarizedRhsRange::*;

pub mod earley;

pub type HistoryId = NonZeroUsize;

// #[derive(Clone, Debug)]
// pub enum HistoryGraph {
//     Nodes {
//         nodes: Vec<HistoryNode>,
//     }, 
//     Earley {
//         earley: Vec<earley::History>,
//     },
// }

// pub enum HistoryKind {
//     Earley,
// }

// impl Default for HistoryGraph {
//     fn default() -> Self {
//         Self::nodes()
//     }
// }

// impl HistoryGraph {
//     pub fn nodes() -> Self {
//         Self::Nodes {
//             nodes: vec![RootHistoryNode::NoOp.into()],
//         }
//     }

//     pub fn earley() -> Self {
//         Self::Earley { earley: vec![earley::History::new(0)] }
//     }

//     // pub fn enable(&mut self, history_kind: HistoryKind) {
//     //     match history_kind {
//     //         HistoryKind::Earley => {
//     //             let mut prev_histories = vec![];
//     //             for node in &self.nodes {
//     //                 prev_histories.push(earley::process_node(node, &prev_histories[..]));
//     //             }
//     //             self.earley = Some(prev_histories);
//     //         }
//     //     }
//     // }

//     pub fn next_id(&mut self) -> HistoryId {
//         match self {
//             Self::Nodes { nodes } => nodes.len(),
//             Self::Earley { earley } => earley.len(),
//         }.try_into().expect("problem with zero length history graph")
//     }

//     pub fn add_history_node(&mut self, node: HistoryNode) -> HistoryId {
//         let result = self.next_id();
//         match self {
//             Self::Nodes { nodes }
//         }
//         if let Some(earley) = self.earley.as_mut() {
//             let result = earley::process_node(&node, &earley[..]);
//             earley.push(result);
//         }
//         self.push(node);
//         result
//     }
// }

// impl ::std::ops::Deref for HistoryGraph {
//     type Target = Vec<HistoryNode>;

//     fn deref(&self) -> &Self::Target {
//         &self.nodes
//     }
// }

// impl ::std::ops::DerefMut for HistoryGraph {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.nodes
//     }
// }

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

pub struct HistoryNodeRhs {
    pub prev: History,
    pub rhs: Vec<Symbol>,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeBinarize {
    pub prev: History,
    pub height: u32,
    pub full_len: usize,
    pub is_top: bool,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeWeight {
    pub prev: History,
    pub weight: f64,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeEliminateNulling {
    pub prev: History,
    pub rhs0: Option<Symbol>,
    pub rhs1: Option<Symbol>,
    pub which: BinarizedRhsRange,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeAssignPrecedence {
    pub prev: History,
    pub looseness: u32,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeRewriteSequence {
    pub prev: History,
    pub top: bool,
    pub rhs: Symbol,
    pub sep: Option<Symbol>,
}

#[derive(Clone, Copy)]
pub struct HistoryNodeSequenceRhs {
    pub prev: History,
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
        process_linked(&LinkedHistoryNode::Binarize {
                height: value.height,
                full_len: value.full_len,
                is_top: value.is_top,
            }, value.prev)
    }
}

impl From<HistoryNodeWeight> for History {
    fn from(value: HistoryNodeWeight) -> Self {
        process_linked(&LinkedHistoryNode::Weight {
            weight: value.weight,
        }, value.prev)
    }
}

impl From<HistoryNodeEliminateNulling> for History {
    fn from(value: HistoryNodeEliminateNulling) -> Self {
        process_linked(&LinkedHistoryNode::EliminateNulling {
            rhs0: value.rhs0,
            rhs1: value.rhs1,
            which: value.which,
        }, value.prev)
    }
}

impl From<HistoryNodeAssignPrecedence> for History {
    fn from(value: HistoryNodeAssignPrecedence) -> Self {
        process_linked(&LinkedHistoryNode::AssignPrecedence {
                looseness: value.looseness,
            }, value.prev)
    }
}

impl From<HistoryNodeRewriteSequence> for History {
    fn from(value: HistoryNodeRewriteSequence) -> Self {
        process_linked(&LinkedHistoryNode::RewriteSequence {
                top: value.top,
                rhs: value.rhs,
                sep: value.sep,
            }, value.prev)
    }
}

impl From<HistoryNodeSequenceRhs> for History {
    fn from(value: HistoryNodeSequenceRhs) -> Self {
        process_linked(&LinkedHistoryNode::SequenceRhs {
                rhs: value.rhs,
            }, value.prev)
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
