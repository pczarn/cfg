#![cfg(feature = "cfg-predict-distance")]

use cfg::Cfg;
use cfg::predict_distance::MinimalDistance;

#[test]
fn test_minimum_distance() {
    let mut cfg = Cfg::new();
    //   0      1  2  3  4  5
    let [start, a, b, c, x, y] = cfg.sym();
    cfg.rule(a)
        .rhs([])
        .rule(start)
        .rhs([a, x, b, c, y])
        .rhs([c])
        .rule(b)
        .rhs([a, a])
        .rhs([a, c])
        .rule(c)
        .rhs([x])
        .rhs([y]);

    let mut minimal_distance = MinimalDistance::new(&cfg);
    let distances = minimal_distance
        .minimal_distances(&[(1, 3)], cfg_predict_distance::DistanceDirection::Forward);
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

    assert_eq!(distances, &expected_distances[..]);
    // for (result, expected) in distances.iter().zip(expected_distances.iter()) {
    //     assert_eq!(result, expected);
    // }
}
