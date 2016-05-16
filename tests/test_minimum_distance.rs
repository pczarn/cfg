extern crate cfg;

use cfg::*;
use cfg::history::{Action, RewriteSequence};
use cfg::prediction::MinimalDistance;

#[derive(Clone, Debug, Eq, PartialEq)]
struct History {
    events: &'static [u32],
}

impl History {
    fn new(slice: &'static [u32]) -> Self {
        History { events: slice }
    }
}

impl Action for History {
    fn no_op(&self) -> Self {
        unimplemented!()
    }
}
impl RewriteSequence for History {
    type Rewritten = Self;

    fn top(&self, _rhs: Symbol, _sep: Option<Symbol>, _new_rhs: &[Symbol]) -> Self {
        unimplemented!()
    }

    fn bottom(&self, _rhs: Symbol, _sep: Option<Symbol>, _new_rhs: &[Symbol]) -> Self {
        unimplemented!()
    }
}

#[test]
fn test_minimum_distance() {
    let mut cfg: Cfg<History> = Cfg::new();
    //   0      1  2  3  4  5
    let (start, a, b, c, x, y) = cfg.sym();
    const POS_3: &'static [u32] = &[3];
    cfg .rule(a)
            .rhs_with_history([], History::new(&[]))
        .rule(start)
            .rhs_with_history([a, x, b, c, y], History::new(POS_3))
            .rhs_with_history([c], History::new(&[]))
        .rule(b)
            .rhs_with_history([a, a], History::new(&[]))
            .rhs_with_history([a, c], History::new(&[]))
        .rule(c)
            .rhs_with_history([x], History::new(&[]))
            .rhs_with_history([y], History::new(&[]));

    let iter = cfg.rules().map(|rule| {
        (rule, rule.history.events.iter().map(|&pos| pos as usize))
    });
    let mut minimal_distance = MinimalDistance::new(&cfg);
    let distances = minimal_distance.minimal_distances(iter);
    // min(x) = min(y) = 1
    // min(b) = 0
    // min(a) = 0
    // min(c) = 1
    let expected_distances = vec![
        vec![Some(0)],
        vec![Some(1), Some(1), Some(0), Some(0), None, None],
        vec![None, None],
        vec![Some(0), Some(0), Some(0)],
        vec![Some(1), Some(1), Some(0)],
        vec![Some(1), Some(0)],
        vec![Some(1), Some(0)],
    ];

    for (result, expected) in distances.iter().zip(expected_distances.iter()) {
        assert_eq!(&result[..], &expected[..]);
    }
}
