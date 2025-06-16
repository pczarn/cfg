use cfg::Cfg;

pub fn grammar() -> Cfg {
    let mut bnf = Cfg::new();
    let [expr, op, num, plus, minus, mul, div] = bnf.sym();
    bnf.rule(expr).rhs([expr, op, expr]).rhs([num]);
    bnf.rule(op).rhs([plus]).rhs([minus]).rhs([mul]).rhs([div]);

    for _ in 0..10 {
        let [sym] = bnf.sym();
        bnf.rule(num).rhs([sym, num]).rhs([sym]);
    }
    bnf.set_roots([expr]);
    bnf
}
