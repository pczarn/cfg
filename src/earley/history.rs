use std::iter;
use optional::Optioned;

use history::*;
use rule::GrammarRule;
use Symbol;

type ExternalOrigin = Option<u32>;
type EventId = Optioned<u32>;
type MinimalDistance = Optioned<u32>;
type NullingEliminated = Option<(Symbol, bool)>;
pub type ExternalDottedRule = (u32, u32);
pub type Event = (EventId, MinimalDistance);

/// Default history.
#[derive(Copy, Clone)]
pub struct BuildHistory {
    num_rules: usize,
}

impl BuildHistory {
    /// Creates default history.
    pub(in super) fn new(num_rules: usize) -> Self {
        BuildHistory { num_rules }
    }
}

impl HistorySource<History> for BuildHistory {
    fn build(&mut self, _lhs: Symbol, rhs: &[Symbol]) -> History {
        // for sequences, rhs.len() will be 1 or 2
        let ret = History::new(self.num_rules as u32, rhs.len());
        self.num_rules += 1;
        ret
    }
}

#[derive(Clone, Default, Debug)]
pub struct History {
    pub dots: Vec<RuleDot>,
    pub origin: ExternalOrigin,
    pub nullable: NullingEliminated,
}

#[derive(Copy, Clone, Debug)]
pub struct RuleDot {
    pub event: Option<(EventId, ExternalDottedRule)>,
    pub distance: MinimalDistance,
}

impl RuleDot {
    fn new(id: u32, pos: usize) -> Self {
        RuleDot {
            event: Some((Optioned::none(), (id, pos as u32))),
            distance: Optioned::none(),
        }
    }

    pub fn none() -> Self {
        RuleDot {
            event: None,
            distance: Optioned::none(),
        }
    }

    pub fn trace(self) -> Option<ExternalDottedRule> {
        self.event.map(|x| x.1)
    }

    pub fn event(self) -> Option<(EventId, ExternalDottedRule)> {
        self.event
    }

    pub fn event_without_tracing(self) -> Event {
        (self.event.and_then(|x| x.0.into()).into(), self.distance)
    }

    pub fn distance(&self) -> MinimalDistance {
      self.distance
    }
}

impl History {
    pub fn new(id: u32, len: usize) -> Self {
        History {
            origin: Some(id),
            dots: (0 .. len + 1).map(|i| RuleDot::new(id, i)).collect(),
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
      self.dots[n]
    }
}

impl Action for History {
    fn no_op(&self) -> Self {
        History::default()
    }
}

impl Binarize for History {
    fn binarize<R>(&self, _rule: &R, depth: usize) -> Self {
        let none = RuleDot::none();
        let dots = if self.dots.is_empty() {
            [none; 3]
        } else {
            let dot_len = self.dots.len();
            if depth == 0 {
                if dot_len == 2 {
                    [self.dots[0], none, self.dots[1]]
                } else if dot_len >= 3 {
                    [self.dots[0], self.dots[dot_len - 2], self.dots[dot_len - 1]]
                } else {
                    [self.dots[0], none, none]
                }
            } else {
                [none, self.dots[dot_len - 2 - depth], none]
            }
        };

        let origin = if depth == 0 {
            self.origin
        } else {
            None
        };

        History {
            origin,
            dots: dots[..].to_vec(),
            nullable: self.nullable,
        }
    }
}

impl EliminateNulling for History {
    fn eliminate_nulling<R>(&self, rule: &R, subset: BinarizedRhsSubset) -> Self where
                R: GrammarRule {
        if let BinarizedRhsSubset::All = subset {
            History {
                origin: self.origin,
                ..History::default()
            }
        } else {
            let right = if let BinarizedRhsSubset::Right = subset { true } else { false };
            let sym = rule.rhs()[right as usize];
            History {
                nullable: Some((sym, right)),
                ..self.clone()
            }
        }
    }
}

#[derive(Copy, Clone)]
enum SymKind {
    Element,
    Separator,
    Other,
}

impl RewriteSequence for History {
    type Rewritten = History;

    fn top(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self {
        let mut bottom = self.bottom(rhs, sep, new_rhs);
        bottom.origin = self.origin;
        bottom
    }

    fn bottom(&self, rhs: Symbol, sep: Option<Symbol>, new_rhs: &[Symbol]) -> Self {
        //  -  sym (1) Sep (2)
        //  -  lhs (1) Sep (2) Rhs (1)
        //  -  lhs (0) Rhs (1)
        // (0) Rhs (1)
        // (0) Rhs (1) Sep (2) Rhs (1)
        // (0) Rhs (1) Rhs (1)
        let syms = new_rhs.iter().map(|&sym| {
            if sym == rhs {
                SymKind::Element
            } else if Some(sym) == sep {
                SymKind::Separator
            } else {
                SymKind::Other
            }
        }).chain(iter::once(SymKind::Other));
        let mut to_left = SymKind::Other;
        let dots = syms.map(|to_right| {
            let dot = match (to_left, to_right) {
                (_, SymKind::Separator) => self.dots[1],
                (SymKind::Separator, _) => self.dots[2],
                (SymKind::Element, _)   => self.dots[1],
                (_, SymKind::Element)   => self.dots[0],
                _ => RuleDot::none()
            };
            to_left = to_right;
            dot
        }).collect();
        History {
            dots,
            ..History::default()
        }
    }
}
