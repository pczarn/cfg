#![cfg(feature = "cfg-classify")]

use std::collections::VecDeque;

use cfg::classify::CfgClassifyExt;
use cfg::{Cfg, Symbol};
use test_case::test_case;

mod support;

#[test]
fn test_binarize() {
    let mut cfg: Cfg = Cfg::new();
    let [start, a, b, c, x, y] = cfg.sym();

    cfg.rule(start)
        .rhs([a, x, b])
        .rhs([c])
        .rule(b)
        .rhs([a, a])
        .rhs([a, c])
        .rule(c)
        .rhs([x])
        .rhs([y])
        .rule(a)
        .rhs([]);

    cfg.set_roots(&[start]);
    cfg.limit_rhs_len(Some(2));

    {
        let mut equivalent = Cfg::new();
        let [start, _a, b, c, x, y, g0] = equivalent.sym();
        equivalent
            .rule(start)
            .rhs([g0, b])
            .rule(g0)
            .rhs([a, x])
            .rule(start)
            .rhs([c])
            .rule(b)
            .rhs([a, a])
            .rhs([a, c])
            .rule(c)
            .rhs([x])
            .rhs([y])
            .rule(a)
            .rhs([]);

        equivalent.set_roots([start]);

        support::assert_eq(&equivalent, &cfg);
    };

    assert!(cfg.usefulness().all_useful());
}

#[test_case(3, 10)]
#[test_case(100, 10_000)]
#[test_case(1000, 10_000)]
#[test_case(423, 10_000)]
fn test_binarize_very_long_rule(num_syms: usize, rhs_len: usize) {
    let mut cfg: Cfg = Cfg::new();
    let start = cfg.next_sym();

    let mut long_rhs = cfg
        .sym_source_mut()
        .generate()
        .take(num_syms)
        .collect::<Vec<_>>();
    long_rhs = long_rhs.iter().cloned().cycle().take(rhs_len).collect();
    cfg.rule(start).rhs(long_rhs);

    cfg.set_roots(&[start]);

    assert!(cfg.usefulness().all_useful());
    cfg.limit_rhs_len(Some(2));
    assert_eq!(cfg.rules().count(), rhs_len - 1);

    let mut equivalent = Cfg::new();
    let start = equivalent.next_sym();

    let mut long_rhs: VecDeque<Symbol> = equivalent
        .sym_source_mut()
        .generate()
        .take(num_syms)
        .collect();
    long_rhs = long_rhs.iter().cloned().cycle().take(rhs_len).collect();
    while long_rhs.len() > 2 {
        let new_sym = equivalent.next_sym();
        let rhs = [long_rhs.pop_front().unwrap(), long_rhs.pop_front().unwrap()];
        equivalent.rule(new_sym).rhs(rhs);
        long_rhs.push_front(new_sym);
    }
    equivalent
        .rule(start)
        .rhs(long_rhs.into_iter().collect::<Vec<_>>());

    equivalent.set_roots([start]);

    support::assert_eq(&equivalent, &cfg);
}
