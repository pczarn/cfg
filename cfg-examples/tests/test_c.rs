use cfg::{
    NamedCfgRule,
    classify::{CfgClassifyExt, recursive::RecursionKind},
    named_cfg_rule,
};
use cfg_examples::c::grammar;

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
        .map(|rec| (rec.rule.named(grammar.sym_source()), rec.recursion))
        .take(2) // TODO full
        .collect();
    let expected_recursive_rules: Vec<(NamedCfgRule, RecursionKind)> = vec![
        (
            named_cfg_rule!(primary_expression ::= lparen expression rparen),
            RecursionKind {
                left: false,
                middle: true,
                right: false,
            },
        ),
        (
            named_cfg_rule!(postfix_expression ::= primary_expression),
            RecursionKind {
                left: true,
                middle: false,
                right: true,
            },
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
        .map(|rec| {
            (
                rec.rule.named(grammar.sym_source()),
                rec.recursion,
                rec.distances,
            )
        })
        .take(2) // TODO full
        .collect();
    let expected_recursive_rules: Vec<(NamedCfgRule, RecursionKind, Option<(usize, usize)>)> = vec![
        (
            named_cfg_rule!(primary_expression ::= lparen expression rparen),
            RecursionKind {
                left: false,
                middle: true,
                right: false,
            },
            Some((0, 1)),
        ),
        (
            named_cfg_rule!(postfix_expression ::= primary_expression),
            RecursionKind {
                left: true,
                middle: false,
                right: true,
            },
            Some((0, 1)),
        ),
    ];
    assert_eq!(actual_recursive_rules, expected_recursive_rules);
}
