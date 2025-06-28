pub mod rule_dot;

use core::iter;

use cfg_symbol::Symbol;

use crate::{
    BinarizedRhsRange, LinkedHistoryNode, RootHistoryNode
};

use rule_dot::RuleDot;

#[cfg_attr(feature = "miniserde", derive(miniserde::Serialize, miniserde::Deserialize))]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct ExternalOrigin { pub id: u32 }
#[cfg_attr(feature = "miniserde", derive(miniserde::Serialize, miniserde::Deserialize))]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct EventId { pub id: u32 }
#[cfg_attr(feature = "miniserde", derive(miniserde::Serialize, miniserde::Deserialize))]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct MinimalDistance { pub distance: u32 }
pub type NullingEliminated = Option<(Symbol, bool)>;
#[cfg_attr(feature = "miniserde", derive(miniserde::Serialize, miniserde::Deserialize))]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct ExternalDottedRule {
    id: u32,
    pos: u32,
}
pub type EventAndDistance = (EventId, MinimalDistance);

impl MinimalDistance {
    pub fn null() -> Self {
        MinimalDistance { distance: !0 }
    }
}

impl EventId {
    pub fn null() -> Self {
        EventId { id: 0 }
    }
}

impl ExternalDottedRule {
    pub fn null() -> Self {
        ExternalDottedRule { id: !0, pos: !0 }
    }
}

impl ExternalOrigin {
    pub fn null() -> Self {
        ExternalOrigin { id: !0 }
    }

    pub fn is_null(&self) -> bool {
        self.id == !0
    }
}

#[derive(Copy, Clone)]
enum SymKind {
    Element,
    Separator,
    Other,
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct History {
    pub dots: [RuleDot; 3],
    pub origin: ExternalOrigin,
    pub nullable: NullingEliminated,
    pub weight: Option<u32>,
    pub sequence: Option<SequenceDetails>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceDetails {
    top: bool,
    rhs: Symbol,
    sep: Option<Symbol>,
}

// pub trait HistoryGraphEarleyExt {
//     fn transpose(&self) -> [impl Iterator<Item = RuleDot>; 3];
// }

// impl HistoryGraphEarleyExt for HistoryGraph {
//     fn transpose(&self) -> [impl Iterator<Item = RuleDot>; 3] {
//         let x = |(history, at): (&History, usize)| history.at_dot(at);
//         [
//             self.earley.as_ref().unwrap().iter().zip(iter::repeat(0)).map(x),
//             self.earley.as_ref().unwrap().iter().zip(iter::repeat(1)).map(x),
//             self.earley.as_ref().unwrap().iter().zip(iter::repeat(2)).map(x),
//         ]
//     }
// }

pub fn process_linked(linked_node: &LinkedHistoryNode, mut prev_history: History) -> History {
    match linked_node {
        LinkedHistoryNode::AssignPrecedence { looseness: _, .. } => prev_history,
        &LinkedHistoryNode::Binarize { height, full_len, is_top, .. } => {
            prev_history.binarize(height, full_len, is_top)
        }
        &LinkedHistoryNode::EliminateNulling {
            which, rhs0, rhs1, ..
        } => prev_history.eliminate_nulling(rhs0, rhs1, which),
        &LinkedHistoryNode::RewriteSequence { top, rhs, sep, .. } => {
            prev_history.sequence = Some(SequenceDetails { top, rhs, sep });
            prev_history
        }
        &LinkedHistoryNode::Weight { weight, .. } => {
            prev_history.weight = Some((weight * 1000f64) as u32);
            prev_history
        }
        &LinkedHistoryNode::SequenceRhs { rhs } => {
            if let Some(sequence_details) = prev_history.sequence {
                let rhs: Vec<_> = rhs.into_iter().flatten().collect();
                prev_history.rewrite_sequence(sequence_details, &rhs[..]);
            }
            prev_history
        }
        // ???
        // &LinkedHistoryNode::Distances { .. } => prev_history,
    }
}

pub(crate) fn process_root(root_node: RootHistoryNode) -> History {
    match root_node {
        RootHistoryNode::NoOp => History::new(!0),
        RootHistoryNode::Rule { lhs: _ } => History::new(!0),
        RootHistoryNode::Origin { origin } => History::new(origin as u32),
    }
}

impl From<RootHistoryNode> for History {
    fn from(value: RootHistoryNode) -> Self {
        process_root(value)
    }
}

impl History {
    pub fn new(id: u32) -> Self {
        // assert!(!ExternalOrigin { id }.is_null());
        History {
            origin: ExternalOrigin { id },
            ..History::default()
        }
    }

    pub fn origin(&self) -> ExternalOrigin {
        self.origin
    }

    pub fn nullable(&self) -> NullingEliminated {
        self.nullable
    }

    pub fn dot(&self, n: usize) -> RuleDot {
        self.dots.get(n).copied().unwrap_or(RuleDot::none())
    }

    fn at_dot(&self, n: usize) -> RuleDot {
        RuleDot::new(self.origin.id, n as u32)
    }

    fn binarize(&self, height: u32, full_len: usize, is_top: bool) -> Self {
        let none = RuleDot::none();
        let dots = if self.dots.is_empty() {
            [none; 3]
        } else {
            if is_top {
                if full_len == 2 {
                    [self.at_dot(0), none, self.at_dot(1)]
                } else if full_len >= 3 {
                    [self.at_dot(0), self.at_dot(full_len - 2), self.at_dot(full_len - 1)]
                } else {
                    [self.at_dot(0), none, none]
                }
            } else {
                [none, self.at_dot(full_len - 2 - height as usize), none]
            }
        };

        let origin = if is_top { self.origin } else { ExternalOrigin::null() };

        History {
            origin,
            dots,
            ..self.clone()
        }
    }

    fn eliminate_nulling(
        &self,
        rhs0: Option<Symbol>,
        rhs1: Option<Symbol>,
        subset: BinarizedRhsRange,
    ) -> Self {
        if let BinarizedRhsRange::All(_) = subset {
            History {
                origin: self.origin,
                ..History::default()
            }
        } else {
            let sym = if let BinarizedRhsRange::Right = subset {
                rhs1.unwrap()
            } else {
                rhs0.unwrap()
            };
            History {
                nullable: Some((sym, BinarizedRhsRange::Right == subset)),
                ..self.clone()
            }
        }
    }

    fn rewrite_sequence(&self, details: SequenceDetails, new_rhs: &[Symbol]) -> Self {
        if details.top {
            self.rewrite_sequence_top(details, new_rhs)
        } else {
            self.rewrite_sequence_bottom(details, new_rhs)
        }
    }

    fn rewrite_sequence_top(&self, details: SequenceDetails, new_rhs: &[Symbol]) -> Self {
        let mut bottom = self.rewrite_sequence_bottom(details, new_rhs);
        bottom.origin = self.origin;
        bottom
    }

    fn rewrite_sequence_bottom(&self, details: SequenceDetails, new_rhs: &[Symbol]) -> Self {
        //  -  sym (1) Sep (2)
        //  -  lhs (1) Sep (2) Rhs (1)
        //  -  lhs (0) Rhs (1)
        // (0) Rhs (1)
        // (0) Rhs (1) Sep (2) Rhs (1)
        // (0) Rhs (1) Rhs (1)
        let syms = new_rhs
            .iter()
            .map(|&sym| {
                if sym == details.rhs {
                    SymKind::Element
                } else if Some(sym) == details.sep {
                    SymKind::Separator
                } else {
                    SymKind::Other
                }
            })
            .chain(iter::once(SymKind::Other));
        let mut to_left = SymKind::Other;
        let mut dots = [RuleDot::none(); 3];
        for (i, to_right) in syms.enumerate() {
            dots[i] = match (to_left, to_right) {
                (_, SymKind::Separator) => self.dots[1],
                (SymKind::Separator, _) => self.dots[2],
                (SymKind::Element, _) => self.dots[1],
                (_, SymKind::Element) => self.dots[0],
                _ => RuleDot::none(),
            };
            to_left = to_right;
        }
        History {
            dots,
            ..History::default()
        }
    }
}
