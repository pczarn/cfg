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
fn test_sequence_1_4() {
    let mut cfg: Cfg = Cfg::new();
    let (start, elem, sep) = cfg.sym();

    cfg.sequence(start).separator(Trailing(sep)).inclusive(1, Some(4)).rhs(elem);
    cfg.rewrite_sequences();

    let mut equiv: Cfg = Cfg::new();
    let (start, elem, sep, g0, g1, g2, g3, g4, g5) = equiv.sym();

    equiv.rule(start).rhs([g0, sep])      // start => (elem sep){1,4}
         .rule(g0).rhs([g1, g2])          // g0  =>  g1 g2  =>  (elem sep){0,3} elem
         .rule(g2).rhs([g3])              // g2  =>  g3 | g4  =>  elem | elem sep elem
                  .rhs([g4])
         .rule(g4).rhs([elem, sep, elem]) // g4 => elem sep elem
         .rule(g3).rhs([elem])            // g3 => elem
         .rule(g1).rhs([g5, sep])         // g1  =>  g5 sep  =>  sep | elem sep | elem sep elem sep
         .rule(g5).rhs([])                // g5  =>  () | g2  =>  () | elem | elem sep elem
                  .rhs([g2]);

    support::assert_eq_rules(equiv.rules(), cfg.rules());
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
