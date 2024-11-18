#![cfg(feature = "cfg-classify")]

mod support;

use cfg::classify::CfgClassifyExt;
use cfg::symbol_bit_matrix::CfgSymbolBitMatrixExt;
use cfg::symbol_bit_matrix::Remap;
use cfg::Cfg;
use cfg::Symbolic;

#[test]
fn test_remap_unused_symbols() {
    let mut cfg: Cfg = Cfg::new();
    let [start, a, x, b, c, _gg, y] = cfg.sym();

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

    cfg.set_roots([start]);

    cfg.remove_unused_symbols();

    {
        let mut equivalent: Cfg = Cfg::new();
        let [start, a, x, b, c, y] = equivalent.sym();
        equivalent
            .rule(start)
            .rhs([a, x, b])
            .rhs([c])
            .rule(b)
            .rhs([a, a])
            .rhs([a, c])
            .rule(c)
            .rhs([x])
            .rule(a)
            .rhs([])
            .rule(c)
            .rhs([y]);
        support::assert_eq_rules(equivalent.rules(), cfg.rules());
    };

    assert!(cfg.usefulness().all_useful());
}

#[test]
fn test_reorder_symbols() {
    let mut cfg: Cfg = Cfg::new();
    //  (0      1  2  3  4  5)
    let [start, a, x, b, c, y] = cfg.sym();

    cfg.rule(start)
        .rhs([a, x, b, c, y])
        .rhs([c])
        .rule(b)
        .rhs([a, a])
        .rhs([a, c])
        .rule(c)
        .rhs([x])
        .rhs([y])
        .rule(a)
        .rhs([]);

    cfg.set_roots([start]);

    let ordering = &[
        /* start => */ 1, /* a => */ 2, /* x => */ 5, /* b => */ 3,
        /* c => */ 4, /* y => */ 6,
    ];
    Remap::new(&mut cfg)
        .reorder_symbols(|left, right| ordering[left.usize()].cmp(&ordering[right.usize()]));

    {
        let mut equivalent: Cfg = Cfg::new();
        //  (0      1  2  3  4  5)
        let [start, a, b, c, x, y] = equivalent.sym();
        equivalent
            .rule(a)
            .rhs([])
            .rule(start)
            .rhs([a, x, b, c, y])
            .rhs([c])
            .rule(b)
            .rhs([a, a])
            .rhs([a, c])
            .rule(c)
            .rhs([x])
            .rhs([y]);
        support::assert_eq_rules(equivalent.rules(), cfg.rules());
    };
    assert!(cfg.usefulness().all_useful());
}

#[test]
fn test_maps() {
    let mut cfg: Cfg = Cfg::new();
    //  (0      1  2  3  4  5)
    let [start, a, x, b, c, y] = cfg.sym();

    cfg.rule(start)
        .rhs([a, x, b, c, y])
        .rhs([c])
        .rule(b)
        .rhs([a, a])
        .rhs([a, c])
        .rule(c)
        .rhs([x])
        .rhs([y])
        .rule(a)
        .rhs([]);

    cfg.set_roots([start]);

    assert!(cfg.usefulness().all_useful());

    let ordering = &[
        /* start => */ 1, /* a => */ 2, /* x => */ 5, /* b => */ 3,
        /* c => */ 4, /* y => */ 6,
    ];

    let mut remap = Remap::new(&mut cfg);
    remap.reorder_symbols(|left, right| ordering[left.usize()].cmp(&ordering[right.usize()]));
    let maps = remap.get_mapping();

    let mut equivalent: Cfg = Cfg::new();
    //  (0      1  2  3  4  5)
    let [start2, a2, b2, c2, x2, y2] = equivalent.sym();

    assert_eq!(maps.to_external, &[start, a, b, c, x, y]);
    assert_eq!(
        maps.to_internal,
        &[
            Some(start2),
            Some(a2),
            Some(x2),
            Some(b2),
            Some(c2),
            Some(y2)
        ]
    );
}
