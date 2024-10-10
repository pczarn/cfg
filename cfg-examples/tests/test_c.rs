use std::num::NonZeroUsize;

use cfg::{classify::CfgClassifyExt, NamedCfgRule};
use cfg_examples::c::{grammar, SYM_NAMES};

#[test]
fn test_useful() {
    let mut grammar = grammar();
    assert!(grammar.usefulness().all_useful());
}

#[test]
fn test_recursive() {
    let grammar = grammar();
    let recursion = grammar.recursion();
    let actual_recursive_rules: Vec<_> = recursion
        .recursive_rules()
        .map(|rule| rule.named(SYM_NAMES))
        .collect();
    let expected_recursive_rules: Vec<NamedCfgRule> = vec![
        NamedCfgRule::new(&["primary_expression", "lparen", "expression", "rparen"], NonZeroUsize::new(10).unwrap()),
    ];
    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}

#[test]
fn test_recursive_distances() {
    let grammar = grammar();
    let recursion = grammar.recursion();
    let actual_recursive_rules: Vec<_> = recursion
        .minimal_distances()
        .map(|cfg_rule_with_distance| (cfg_rule_with_distance.prediction_distance, cfg_rule_with_distance.completion_distance, cfg_rule_with_distance.rule.named(SYM_NAMES)))
        .collect();
    let expected_recursive_rules: Vec<(usize, usize, NamedCfgRule)> = vec![];
    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}
