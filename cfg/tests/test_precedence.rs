#![cfg(feature = "cfg-classify")]

mod support;

use cfg::precedenced_rule::Associativity::*;
use cfg::Cfg;
use cfg_classify::CfgClassifyExt;

#[test]
fn test_simple_precedence() {
    let mut cfg: Cfg = Cfg::new();
    let [start, top, num, var] = cfg.sym();
    let [l_paren, r_paren, exp, mul, div, plus, minus, eq] = cfg.sym();

    // top ::=
    //      num
    //   |  var
    //   |  assoc:Group '(' top ')'
    //   |> '-' top
    //   |> assoc:Right top '^' top
    //   |> top '*' top
    //   |  top '/' top
    //   |> top '+' top
    //   |  top '-' top
    //   |> var '=' top

    cfg.rule(start)
        .rhs([top])
        .precedenced_rule(top)
        .rhs([num])
        .rhs([var])
        .associativity(Group)
        .rhs([l_paren, top, r_paren])
        .lower_precedence()
        .rhs([minus, top])
        .associativity(Right)
        .rhs([top, exp, top])
        .lower_precedence()
        .rhs([top, mul, top])
        .rhs([top, div, top])
        .lower_precedence()
        .rhs([top, plus, top])
        .rhs([top, minus, top])
        .lower_precedence()
        .rhs([var, eq, top])
        .finalize();

    cfg.set_roots(&[start]);

    let mut equivalent: Cfg = Cfg::new();
    let [start, top, num, var] = equivalent.sym();
    let [l_paren, r_paren, exp, mul, div, plus, minus, eq] = equivalent.sym();
    let [g4, g3, g2, g1, g0] = equivalent.sym();

    // Order is significant.
    equivalent
        .rule(start)
        .rhs([top])
        .rule(g4)
        .rhs([num])
        .rhs([var])
        .rule(g3)
        .rhs([g4])
        .rhs([minus, g3])
        .rhs([g4, exp, g3])
        .rule(g2)
        .rhs([g3])
        .rhs([g2, mul, g3])
        .rhs([g2, div, g3])
        .rule(g1)
        .rhs([g2])
        .rhs([g1, plus, g2])
        .rhs([g1, minus, g2])
        .rule(g0)
        .rhs([g1])
        .rhs([var, eq, g0])
        .rule(g4)
        .rhs([l_paren, g0, r_paren])
        .rule(top)
        .rhs([g0]);

    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(cfg.usefulness().all_useful());
}

#[test]
fn test_ternary_quaternary() {
    let mut cfg: Cfg = Cfg::new();
    let [start, top, num] = cfg.sym();
    let [ternary_op, quaternary_op, sep] = cfg.sym();

    cfg.rule(start)
        .rhs([top])
        .precedenced_rule(top)
        .rhs([num])
        .lower_precedence()
        .associativity(Right)
        .rhs([top, ternary_op, top, sep, top])
        .associativity(Right)
        .rhs([top, quaternary_op, top, sep, top, sep, top])
        .finalize();

    cfg.set_roots(&[start]);

    let mut equivalent: Cfg = Cfg::new();
    let [start, top, num] = equivalent.sym();
    let [ternary_op, quaternary_op, sep] = equivalent.sym();
    let [g1, g0] = equivalent.sym();

    // Order is significant.
    equivalent
        .rule(start)
        .rhs([top])
        .rule(g1)
        .rhs([num])
        .rule(g0)
        .rhs([g1])
        .rhs([g1, ternary_op, g1, sep, g0])
        .rhs([g1, quaternary_op, g1, sep, g1, sep, g0])
        .rule(top)
        .rhs([g0]);

    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(cfg.usefulness().all_useful());
}
