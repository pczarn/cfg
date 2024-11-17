use std::num::NonZeroUsize;

use cfg::{
    classify::{CfgClassifyExt, RecursionKind},
    NamedCfgRule,
};
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
        .map(|rec| (rec.rule.named(SYM_NAMES), rec.recursion))
        .take(2) // TODO full
        .collect();
    let expected_recursive_rules: Vec<(NamedCfgRule, RecursionKind)> = vec![
        (
            NamedCfgRule::new(
                &["primary_expression", "lparen", "expression", "rparen"],
                NonZeroUsize::new(10).unwrap(),
            ),
            RecursionKind::Middle,
        ),
        (
            NamedCfgRule::new(
                &["postfix_expression", "primary_expression"],
                NonZeroUsize::new(12).unwrap(),
            ),
            RecursionKind::All,
        ),
    ];
    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}

#[test]
fn test_recursive_distances() {
    let grammar = grammar();
    let recursion = grammar.recursion();
    let actual_recursive_rules: Vec<_> = recursion
        .minimal_distances()
        .map(|rec| (rec.rule.named(SYM_NAMES), rec.recursion, rec.distances))
        .take(2) // TODO full
        .collect();
    let expected_recursive_rules: Vec<(NamedCfgRule, RecursionKind, Option<(usize, usize)>)> = vec![
        (
            NamedCfgRule::new(
                &["primary_expression", "lparen", "expression", "rparen"],
                NonZeroUsize::new(10).unwrap(),
            ),
            RecursionKind::Middle,
            Some((0, 1)),
        ),
        (
            NamedCfgRule::new(
                &["postfix_expression", "primary_expression"],
                NonZeroUsize::new(12).unwrap(),
            ),
            RecursionKind::All,
            Some((0, 1)),
        ),
    ];
    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}
