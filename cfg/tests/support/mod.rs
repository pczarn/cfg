use cfg::{Cfg, CfgRule};

pub fn assert_eq_rules<'a, 'b, I, J>(i: I, j: J)
where
    I: Iterator<Item = &'a CfgRule>,
    J: Iterator<Item = &'b CfgRule>,
{
    let rules_i = i
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();
    let rules_j = j
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();

    assert_eq!(rules_i, rules_j, "Grammar rules expected to be equal");
}

pub fn assert_eq(left: &Cfg, right: &Cfg) {
    assert_eq_rules(left.rules(), right.rules());
    for root in left.roots() {
        // get root equal
    }
}
