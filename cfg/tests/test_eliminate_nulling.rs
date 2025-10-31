use cfg::Cfg;
use cfg::classify::CfgClassifyExt;

mod support;

#[test]
fn test_binarize_and_eliminate_nulling_rules_history() {
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
    let nulling = cfg.binarize_and_eliminate_nulling_rules();

    {
        let mut equivalent_nulling = Cfg::new();
        let [_start, a, b, _c, _x, _y, _g0] = equivalent_nulling.sym();
        equivalent_nulling.rule(a).rhs([]);
        equivalent_nulling.rule(b).rhs([a, a]);

        support::assert_eq(&equivalent_nulling, &nulling);
    }

    {
        let mut equivalent = Cfg::new();
        let [start, _a, b, c, x, y, g0] = equivalent.sym();
        equivalent
            .rule(start)
            .rhs([c])
            .rhs([g0])
            .rhs([g0, b])
            .rule(b)
            .rhs([a]) // TODO: deduplicate this
            .rhs([a]) // TODO: deduplicate this
            .rhs([a, a])
            .rhs([a, c])
            .rhs([c])
            .rule(c)
            .rhs([x])
            .rhs([y])
            .rule(g0)
            .rhs([a, x])
            .rhs([x]);

        equivalent.set_roots([start]);

        support::assert_eq(&equivalent, &cfg);
    };

    assert!(cfg.usefulness().all_useful());
}
