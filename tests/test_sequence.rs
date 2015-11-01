extern crate cfg;

mod support;

use cfg::*;
use cfg::sequence::Separator::*;
use cfg::usefulness::Usefulness;

#[test]
fn test_sequence() {
    let mut cfg: Cfg = Cfg::new();
    let (start, elem, sep) = cfg.sym();

    cfg.sequence(start).separator(Trailing(sep)).inclusive(1, Some(1)).rhs(elem);
    cfg.rewrite_sequences();

    let mut equivalent: Cfg = Cfg::new();
    let (start, elem, sep, g0) = equivalent.sym();

    equivalent.rule(start).rhs([g0, sep])
              .rule(g0).rhs([elem]);

    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(Usefulness::new(&mut cfg).reachable([start]).all_useful());
}

#[test]
fn test_nulling_sequence() {
    let mut cfg: Cfg = Cfg::new();
    let (start, elem) = cfg.sym();

    cfg.sequence(start).inclusive(0, Some(0)).rhs(elem);
    cfg.rewrite_sequences();

    let mut equivalent: Cfg = Cfg::new();
    let start = equivalent.sym();

    equivalent.rule(start).rhs([]);

    support::assert_eq_rules(equivalent.rules(), cfg.rules());
    assert!(Usefulness::new(&mut cfg).reachable([start]).all_useful());
}

#[test]
fn test_sequence_combinations() {
    for i in 0..50 {
        let mut cfg: Cfg = Cfg::new();
        let (start, elem) = cfg.sym();

        cfg.sequence(start).inclusive(i, Some(99)).rhs(elem);
        cfg.rewrite_sequences();

        assert!(Usefulness::new(&mut cfg).reachable([start]).all_useful());
    }
}
