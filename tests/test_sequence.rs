extern crate cfg;

mod support;

use cfg::*;
use cfg::sequence::Separator::*;
use cfg::usefulness::Usefulness;

#[test]
fn test_sequence() {
    let mut cfg: Cfg = Cfg::new();
    let start = cfg.start_sym();
    let (elem, sep) = cfg.sym();

    cfg.sequence(start).separator(Trailing(sep)).rhs(elem, 1..2);
    cfg.rewrite_sequences();

    let mut equivalent: Cfg = Cfg::new();
    let start = equivalent.start_sym();
    let (elem, sep, g0) = equivalent.sym();

    equivalent.rule(start).rhs([g0, sep])
              .rule(g0).rhs([elem]);

    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(!Usefulness::new(&mut cfg).has_useless_rules());
}

#[test]
fn test_nulling_sequence() {
    let mut cfg: Cfg = Cfg::new();
    let start = cfg.start_sym();
    let elem = cfg.sym();

    cfg.sequence(start).rhs(elem, 0..1);
    cfg.rewrite_sequences();

    let mut equivalent: Cfg = Cfg::new();
    let start = equivalent.start_sym();

    equivalent.rule(start).rhs([]);

    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(!Usefulness::new(&mut cfg).has_useless_rules());
}

#[test]
fn test_sequence_combinations() {
    for i in 0..50 {
        let mut cfg: Cfg = Cfg::new();
        let start = cfg.start_sym();
        let elem = cfg.sym();

        cfg.sequence(start).rhs(elem, i..100);
        cfg.rewrite_sequences();

        assert!(!Usefulness::new(&mut cfg).has_useless_rules());
    }
}
