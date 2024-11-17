#![cfg(feature = "cfg-classify")]

use std::num::NonZeroUsize;
use std::rc::Rc;

#[cfg(feature = "ll")]
use cfg::classify::{LlNonterminalClass, LlParseTable};
use cfg::classify::{Lr0FsmBuilder, Lr0Item, Lr0Items, Lr0Node};
use cfg::{Cfg, CfgRule};
use cfg_classify::CfgClassifyExt;
use cfg_classify::RecursiveRule;

use std::collections::BTreeMap;

#[cfg(feature = "ll")]
#[test]
fn test_ll_classification() {
    let mut cfg = Cfg::new();
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

    let classification = LlParseTable::new(&cfg).classify();
    let classes = classification.classes();

    let mut map = BTreeMap::new();

    map.insert(start, LlNonterminalClass::ContextFree);
    map.insert(a, LlNonterminalClass::Ll1);
    map.insert(b, LlNonterminalClass::Ll1);
    map.insert(c, LlNonterminalClass::Ll1);

    assert_eq!(classes, &map);
}

#[cfg(feature = "ll")]
#[test]
fn test_ll_transitive_classification() {
    let mut cfg: Cfg = Cfg::new();
    let [start, a, b, x, y] = cfg.sym();

    cfg.rule(start)
        .rhs([a])
        .rule(a)
        .rhs([x, b])
        .rule(b)
        .rhs([x, y])
        .rhs([x]);

    let classification = LlParseTable::new(&cfg).classify();
    let classes = classification.classes();

    let mut map = BTreeMap::new();

    map.insert(start, LlNonterminalClass::ContextFree);
    map.insert(a, LlNonterminalClass::ContextFree);
    map.insert(b, LlNonterminalClass::ContextFree);

    assert_eq!(classes, &map);
}

#[test]
fn test_lr0() {
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

    let lr0_fsm = Lr0FsmBuilder::new(&mut cfg).make_lr0_fsm(start);

    let mut items = Lr0Items {
        map: BTreeMap::new(),
    };

    items.map.insert(
        0,
        Lr0Item {
            rhs: vec![a, x, b],
            dot: 0,
        },
    );
    items.map.insert(
        1,
        Lr0Item {
            rhs: vec![c],
            dot: 0,
        },
    );
    items.map.insert(
        4,
        Lr0Item {
            rhs: vec![x],
            dot: 0,
        },
    );
    items.map.insert(
        5,
        Lr0Item {
            rhs: vec![y],
            dot: 0,
        },
    );
    items.map.insert(
        6,
        Lr0Item {
            rhs: vec![],
            dot: 0,
        },
    );
    items.map.insert(
        7,
        Lr0Item {
            rhs: vec![start],
            dot: 0,
        },
    );

    let mut node_0 = Lr0Node {
        items: Rc::new(items),
        link: BTreeMap::new(),
    };

    node_0.link.insert(x, 1);
    node_0.link.insert(y, 2);

    let mut items = Lr0Items {
        map: BTreeMap::new(),
    };

    items.map.insert(
        4,
        Lr0Item {
            rhs: vec![x],
            dot: 1,
        },
    );

    let node_1 = Lr0Node {
        items: Rc::new(items),
        link: BTreeMap::new(),
    };

    let mut items = Lr0Items {
        map: BTreeMap::new(),
    };

    items.map.insert(
        5,
        Lr0Item {
            rhs: vec![y],
            dot: 1,
        },
    );

    let node_2 = Lr0Node {
        items: Rc::new(items),
        link: BTreeMap::new(),
    };

    let nodes = vec![node_0, node_1, node_2];

    assert_eq!(nodes, lr0_fsm);
}

#[test]
fn test_recursive() {
    let mut cfg = Cfg::new();

    let [start, foo, bar] = cfg.sym();

    cfg.rule(start).rhs([foo, foo]);

    cfg.rule(foo).rhs([bar]).rhs([foo, bar]);

    let rec_rule = CfgRule {
        lhs: 1u32.into(),
        rhs: vec![1u32.into(), 2u32.into()].into(),
        history_id: NonZeroUsize::new(6).unwrap(),
    };

    let expected_recursive_rules: Vec<RecursiveRule> = vec![RecursiveRule {
        rule: &rec_rule,
        recursion: cfg_classify::RecursionKind::Left,
        distances: None,
    }];

    let recursion = cfg.recursion();

    let actual_recursive_rules = recursion.recursive_rules().collect::<Vec<_>>();

    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}

#[test]
fn test_recursive_right_rec() {
    let mut cfg = Cfg::new();

    let [start, foo, bar, buzz] = cfg.sym();

    cfg.rule(start).rhs([foo, foo]);

    cfg.rule(foo).rhs([bar]).rhs([bar, bar, buzz, foo]);

    let rec_rule = CfgRule {
        lhs: 1u32.into(),
        rhs: vec![2u32.into(), 2u32.into(), 3u32.into(), 1u32.into()].into(),
        history_id: NonZeroUsize::new(6).unwrap(),
    };

    let expected_recursive_rules: Vec<RecursiveRule> = vec![RecursiveRule {
        rule: &rec_rule,
        recursion: cfg_classify::RecursionKind::Right,
        distances: None,
    }];

    let recursion = cfg.recursion();

    let actual_recursive_rules = recursion.recursive_rules().collect::<Vec<_>>();

    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}
