#![allow(dead_code)]

use cfg::{Cfg, CfgRule};

pub fn eq_rules<'a, 'b>(
    i: impl Iterator<Item = &'a CfgRule>,
    j: impl Iterator<Item = &'b CfgRule>,
) -> bool {
    let mut rules_i = i
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();
    let mut rules_j = j
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();

    rules_i.sort();
    rules_j.sort();

    if rules_i != rules_j {
        eprintln!("Left:");
        eprintln!("{:?}", rules_i);
        eprintln!("Right:");
        eprintln!("{:?}", rules_j);
    }

    rules_i == rules_j
}

pub fn assert_eq_rules<'a, 'b>(
    i: impl Iterator<Item = &'a CfgRule>,
    j: impl Iterator<Item = &'b CfgRule>,
) {
    assert!(eq_rules(i, j), "Rules expected to be equal");
}

pub fn assert_eq(left: &Cfg, right: &Cfg) {
    if !eq_rules(left.rules(), right.rules()) {
        let mut left_sorted = left.clone();
        let mut right_sorted = right.clone();
        left_sorted.sort();
        right_sorted.sort();
        eprintln!(
            "{}\n{}",
            left_sorted.stringify_to_bnf(),
            right_sorted.stringify_to_bnf()
        );
        panic!("Rules expected to be equal");
    }
    let mut roots_i = left.roots().to_vec();
    let mut roots_j = right.roots().to_vec();
    roots_i.sort();
    roots_j.sort();
    assert_eq!(roots_i, roots_j, "Grammar roots expected to be equal");
}
