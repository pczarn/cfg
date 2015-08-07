extern crate cfg;

mod support;

use cfg::{Cfg, ContextFree, ContextFreeRef, GrammarRule, SymbolSource};
use cfg::cycles::Cycles;

#[test]
fn test_remove_cycles() {
    let mut cfg: Cfg = Cfg::new();
    let start = cfg.start_sym();
    let (a, b, c, d) = cfg.sym();

    cfg.rule(start).rhs([a])
       .rule(a).rhs([b])
       .rule(b).rhs([c])
       .rule(c).rhs([d])
       .rule(d).rhs([a]);

    let mut equivalent: Cfg = Cfg::new();
    let start = equivalent.start_sym();
    let a = equivalent.sym();

    // Order is significant.
    equivalent.rule(start).rhs([a]);
    {
        let mut cycles = Cycles::new(&mut cfg);
        let lhss: Vec<_> = cycles.cycle_participants().map(|rule| rule.lhs()).collect();
        assert_eq!(lhss, &[a, b, c, d]);
        cycles.remove_cycles();
    };
    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(!Cycles::new(&mut cfg).has_cycles());
}

#[test]
fn test_rewrite_cycles() {
    let mut cfg: Cfg = Cfg::new();
    let start = cfg.start_sym();
    let (first, second) = cfg.sym();

    cfg.rule(start).rhs([second])
       .rule(first).rhs([second])
       .rule(second).rhs([first]);

    let mut equivalent: Cfg = Cfg::new();
    let start = equivalent.start_sym();
    let first = equivalent.sym();

    // Order is significant.
    equivalent.rule(start).rhs([first]);

    Cycles::new(&mut cfg).rewrite_cycles();
    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(!Cycles::new(&mut cfg).has_cycles());
}

#[test]
fn test_cycle_branch() {
    let mut cfg: Cfg = Cfg::new();
    let start = cfg.start_sym();
    let (a, b, c, d) = cfg.sym();

    cfg.rule(start).rhs([a])
       .rule(a).rhs([b])
       .rule(b).rhs([c])
       .rule(c).rhs([a])
       .rule(c).rhs([d]);

    let mut equivalent: Cfg = Cfg::new();
    let start = equivalent.start_sym();
    let (a, _, _, d) = equivalent.sym();

    // Order is significant.
    equivalent.rule(start).rhs([a])
              .rule(a).rhs([d]);
    {
        let mut cycles = Cycles::new(&mut cfg);
        let lhss: Vec<_> = cycles.cycle_participants().map(|rule| rule.lhs()).collect();
        assert_eq!(lhss, &[a, b, c]);
        cycles.rewrite_cycles();
    };
    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(!Cycles::new(&mut cfg).has_cycles());
}
