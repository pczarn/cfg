#![allow(dead_code)]

use cfg::{Cfg, CfgRule};

pub fn assert_eq_rules<'a, 'b>(
    i: impl Iterator<Item = &'a CfgRule>,
    j: impl Iterator<Item = &'b CfgRule>,
) {
    let mut rules_i = i
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();
    let mut rules_j = j
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();

    rules_i.sort();
    rules_j.sort();

    assert_eq!(rules_i, rules_j, "Grammar rules expected to be equal");
}

pub fn assert_eq(left: &Cfg, right: &Cfg) {
    assert_eq_rules(left.rules(), right.rules());
    let mut roots_i = left.roots().to_vec();
    let mut roots_j = right.roots().to_vec();
    roots_i.sort();
    roots_j.sort();
    assert_eq!(roots_i, roots_j, "Grammar roots expected to be equal");
}
