#![cfg(feature = "cfg-predict-sets")]

use cfg::Cfg;
use cfg_predict_sets::{sets::PerSymbolSetVal, FirstSets, PredictSets};

use std::collections::BTreeMap;

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

    cfg.set_roots([start]);

    let collector = FirstSets::new(&cfg);
    let sets = collector.predict_sets();

    let mut map = BTreeMap::new();
    let start_set = PerSymbolSetVal {
        has_none: false,
        list: vec![x, y],
    };
    let a_set = PerSymbolSetVal {
        has_none: true,
        list: vec![],
    };
    let b_set = PerSymbolSetVal {
        has_none: true,
        list: vec![x, y],
    };
    let c_set = PerSymbolSetVal {
        has_none: false,
        list: vec![x, y],
    };

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
    let start_set = PerSymbolSetVal {
        has_none: false,
        list: vec![x],
    };
    let a_set = PerSymbolSetVal {
        has_none: true,
        list: vec![],
    };
    let b_set = PerSymbolSetVal {
        has_none: true,
        list: vec![x, y],
    };
    let c_set = PerSymbolSetVal {
        has_none: false,
        list: vec![x, y],
    };

    map.insert(start, start_set);
    map.insert(a, a_set);
    map.insert(b, b_set);
    map.insert(c, c_set);

    assert_eq!(sets, &map);
}
