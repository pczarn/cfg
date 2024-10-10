use cfg::Cfg;

pub fn grammar() -> Cfg {
    let mut bnf = Cfg::new();
    let [expr, op, num, plus, minus, mul, div] = bnf.sym();
    bnf.rule(expr).rhs([expr, op, expr]).rhs([num]);
    bnf.rule(op).rhs([plus]).rhs([minus]).rhs([mul]).rhs([div]);

    for sym in bnf.sym::<10>() {
        bnf.rule(num).rhs([sym, num]).rhs([sym]);
    }
    bnf.set_roots(&[expr]);
    bnf
}

#[macro_export]
macro_rules! ambiguous_arith_rhs_elem {
    ('+') => {
        0
    };
    ('-') => {
        1
    };
    ('*') => {
        2
    };
    ('/') => {
        3
    };
    ('0') => {
        4
    };
    ('1') => {
        5
    };
    ('2') => {
        6
    };
    ('3') => {
        7
    };
    ('4') => {
        8
    };
    ('5') => {
        9
    };
    ('6') => {
        10
    };
    ('7') => {
        11
    };
    ('8') => {
        12
    };
    ('9') => {
        13
    };
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! ambiguous_arith {
    ($($e:tt)+) => (
        &[$(ambiguous_arith_rhs_elem!($e) + 3,)+]
    )
}
