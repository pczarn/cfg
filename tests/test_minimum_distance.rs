extern crate cfg;

use std::num::NonZeroUsize;

use cfg::history::LinkedHistoryNode;
use cfg::prediction::MinimalDistance;
use cfg::prelude::*;

fn empty() -> LinkedHistoryNode {
    LinkedHistoryNode::Distances { events: vec![] }
}

fn distances(elems: &[u32]) -> LinkedHistoryNode {
    LinkedHistoryNode::Distances {
        events: elems.to_vec(),
    }
}

#[test]
fn test_minimum_distance() {
    let mut cfg = Cfg::new();
    //   0      1  2  3  4  5
    let (start, a, b, c, x, y) = cfg.sym();
    const POS_3: &'static [u32] = &[3];
    cfg.rule(a)
        .rhs_with_linked_history([], empty())
        .rule(start)
        .rhs_with_linked_history([a, x, b, c, y], distances(POS_3))
        .rhs_with_linked_history([c], empty())
        .rule(b)
        .rhs_with_linked_history([a, a], empty())
        .rhs_with_linked_history([a, c], empty())
        .rule(c)
        .rhs_with_linked_history([x], empty())
        .rhs_with_linked_history([y], empty());

    let mut minimal_distance = MinimalDistance::new(&cfg);
    let distances = minimal_distance.minimal_distances();
    // min(x) = min(y) = 1
    // min(b) = 0
    // min(a) = 0
    // min(c) = 1
    let expected_distances = vec![
        (NonZeroUsize::new(3).unwrap(), vec![Some(0)]),
        (
            NonZeroUsize::new(6).unwrap(),
            vec![Some(1), Some(1), Some(0), Some(0), None, None],
        ),
        (NonZeroUsize::new(9).unwrap(), vec![None, None]),
        (
            NonZeroUsize::new(12).unwrap(),
            vec![Some(0), Some(0), Some(0)],
        ),
        (
            NonZeroUsize::new(15).unwrap(),
            vec![Some(1), Some(1), Some(0)],
        ),
        (NonZeroUsize::new(18).unwrap(), vec![Some(1), Some(0)]),
        (NonZeroUsize::new(21).unwrap(), vec![Some(1), Some(0)]),
    ];

    assert_eq!(distances, &expected_distances[..]);
    // for (result, expected) in distances.iter().zip(expected_distances.iter()) {
    //     assert_eq!(result, expected);
    // }
}
