use cfg::*;

pub fn assert_eq_rules<R, I, J>(i: I, j: J)
where
    R: GrammarRule,
    I: Iterator<Item = R>,
    J: Iterator<Item = R>,
{
    let rules_i = i
        .map(|rule| (rule.lhs(), rule.rhs().to_vec()))
        .collect::<Vec<_>>();
    let rules_j = j
        .map(|rule| (rule.lhs(), rule.rhs().to_vec()))
        .collect::<Vec<_>>();

    assert_eq!(rules_i, rules_j);
}
