use cfg::{Cfg, RuleContainer};
use cfg_predict::{FirstSets, PredictSets};

use std::collections::{BTreeMap, BTreeSet};

#[test]
fn test_simple_first_sets() {
    let mut cfg: Cfg = Cfg::new();
    let [start, a, x, b, c, y] = cfg.sym();

    cfg.rule(start)
        .rhs([a, x, b])
        .rhs([c])
        .rule(b)
        .rhs([a, a])
        .rhs([a, c])
        .rule(c)
        .rhs([x])
        .rhs([y])
        .rule(a)
        .rhs([]);
    let collector = FirstSets::new(&cfg);
    let sets = collector.predict_sets();

    let mut map = BTreeMap::new();
    let mut start_set = BTreeSet::new();
    start_set.insert(Some(x));
    start_set.insert(Some(y));
    let mut a_set = BTreeSet::new();
    a_set.insert(None);
    let mut b_set = BTreeSet::new();
    b_set.insert(None);
    b_set.insert(Some(x));
    b_set.insert(Some(y));
    let mut c_set = BTreeSet::new();
    c_set.insert(Some(x));
    c_set.insert(Some(y));

    map.insert(start, start_set);
    map.insert(a, a_set);
    map.insert(b, b_set);
    map.insert(c, c_set);

    assert_eq!(sets, &map);
}

#[test]
fn test_simple_first_sets_altered() {
    let mut cfg: Cfg = Cfg::new();
    let [start, a, x, b, c, y] = cfg.sym();

    cfg.rule(start)
        .rhs([a, x, b])
        .rule(b)
        .rhs([a, a])
        .rhs([a, c])
        .rule(c)
        .rhs([x])
        .rhs([y])
        .rule(a)
        .rhs([]);
    let collector = FirstSets::new(&cfg);
    let sets = collector.predict_sets();

    let mut map = BTreeMap::new();
    let mut start_set = BTreeSet::new();
    start_set.insert(Some(x));
    let mut a_set = BTreeSet::new();
    a_set.insert(None);
    let mut b_set = BTreeSet::new();
    b_set.insert(None);
    b_set.insert(Some(x));
    b_set.insert(Some(y));
    let mut c_set = BTreeSet::new();
    c_set.insert(Some(x));
    c_set.insert(Some(y));

    map.insert(start, start_set);
    map.insert(a, a_set);
    map.insert(b, b_set);
    map.insert(c, c_set);

    assert_eq!(sets, &map);
}
