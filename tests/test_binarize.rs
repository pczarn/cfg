extern crate cfg;

mod support;

use cfg::*;
use cfg::usefulness::Usefulness;

#[test]
fn test_binarize() {
    let mut cfg: Cfg = Cfg::new();
    let (start, a, b, c, x, y) = cfg.sym();

    cfg.rule(start).rhs([a, x, b]).rhs([c])
       .rule(b).rhs([a, a]).rhs([a, c])
       .rule(c).rhs([x]).rhs([y])
       .rule(a).rhs([]);

    let mut cfg = cfg.binarize();

    {
        let mut equivalent = BinarizedCfg::new();
        let (start, a, b, c, x, y, g0) = equivalent.sym();

        equivalent.rule(start).rhs([g0, b])
                  .rule(g0).rhs([a, x])
                  .rule(start).rhs([c])
                  .rule(b).rhs([a, a]).rhs([a, c])
                  .rule(c).rhs([x]).rhs([y])
                  .rule(a).rhs([]);
        support::assert_eq_rules(equivalent.rules(), cfg.rules());
    };

    let nulling = cfg.eliminate_nulling_rules();

    {
        let mut equivalent = BinarizedCfg::new();
        let (start, _, _, c, x, y, g0) = equivalent.sym();
        equivalent.rule(start).rhs([g0, b]).rhs([c])
                  .rule(c).rhs([x]).rhs([y])
                  .rule(start).rhs([g0])
                  .rule(g0).rhs([x])
                  .rule(b).rhs([c]);
        support::assert_eq_rules(equivalent.rules(), cfg.rules());
    };
    {
        let mut equivalent_nulling = BinarizedCfg::new();
        let (_, a, b) = equivalent_nulling.sym();
        equivalent_nulling.rule(a).rhs([])
                          .rule(b).rhs([a, a]);
        support::assert_eq_rules(equivalent_nulling.rules(), nulling.rules());
    };

    assert!(Usefulness::new(&mut cfg).reachable([start]).all_useful());
}

#[test]
fn test_binarize_very_long_rule() {
    const RULE_COUNT: usize = 10_000;

    let mut cfg: Cfg = Cfg::new();
    let start = cfg.sym();

    let mut long_rhs = cfg.generate().take(100).collect::<Vec<_>>();
    long_rhs = long_rhs.iter().cloned().cycle().take(RULE_COUNT).collect();
    cfg.rule(start).rhs(long_rhs);

    assert!(Usefulness::new(&mut cfg).reachable([start]).all_useful());
    let cfg = cfg.binarize();
    assert_eq!(cfg.rules().count(), RULE_COUNT - 1);

    let mut equivalent = BinarizedCfg::new();
    let start = equivalent.sym();

    let mut long_rhs = equivalent.generate().take(100).collect::<Vec<_>>();
    long_rhs = long_rhs.iter().cloned().cycle().take(RULE_COUNT).collect();
    equivalent.rule(start).rhs(long_rhs);
    support::assert_eq_rules(equivalent.rules(), cfg.rules());
}
