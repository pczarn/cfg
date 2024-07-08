use cfg::rule::RuleRef;

pub fn assert_eq_rules<'a, 'b, I, J>(i: I, j: J)
where
    I: Iterator<Item = RuleRef<'a>>,
    J: Iterator<Item = RuleRef<'b>>,
{
    let rules_i = i
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();
    let rules_j = j
        .map(|rule| (rule.lhs, rule.rhs.to_vec()))
        .collect::<Vec<_>>();

    assert_eq!(rules_i, rules_j);
}
