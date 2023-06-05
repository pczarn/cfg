use cfg::earley::Grammar;
#[cfg(feature = "generation")]
use cfg::generate::weighted::history::WeightedHistory;
#[cfg(feature = "generation")]
use cfg::generate::weighted::WeightedGrammar;

pub fn grammar() -> Grammar {
    let mut bnf = Grammar::new();
    let (sum, product, factor, number, plus, minus, mul, div, lparen, rparen) = bnf.sym();
    bnf.rule(sum)
        .rhs([sum, plus, product])
        .rhs([sum, minus, product])
        .rhs([product])
        .rule(product)
        .rhs([product, mul, factor])
        .rhs([product, div, factor])
        .rhs([factor])
        .rule(factor)
        .rhs([lparen, sum, rparen])
        .rhs([number]);
    for _ in 0..10 {
        let sym = bnf.sym();
        bnf.rule(number).rhs(&[sym, number]).rhs(&[sym]);
    }
    bnf.set_start(sum);
    bnf
}

#[cfg(feature = "generation")]
pub fn weighted_grammar() -> WeightedGrammar<u32> {
    use cfg::earley::history::History;
    let mut bnf = WeightedGrammar::new();
    let (sum, product, factor, number, plus, minus, mul, div, lparen, rparen) = bnf.sym();
    let weighted_hist =
        |weight, len| WeightedHistory::with_history_and_weight(History::new(0, len), weight);
    bnf.rule(sum)
        .history(weighted_hist(1, 3))
        .rhs([sum, plus, product])
        .history(weighted_hist(1, 3))
        .rhs([sum, minus, product])
        .history(weighted_hist(3, 1))
        .rhs([product])
        .rule(product)
        .history(weighted_hist(1, 3))
        .rhs([product, mul, factor])
        .history(weighted_hist(1, 3))
        .rhs([product, div, factor])
        .history(weighted_hist(3, 1))
        .rhs([factor])
        .rule(factor)
        .history(weighted_hist(1, 3))
        .rhs([lparen, sum, rparen])
        .history(weighted_hist(3, 1))
        .rhs([number]);
    for _ in 0..10 {
        let sym = bnf.sym();
        bnf.rule(number).rhs(&[sym, number]).rhs(&[sym]);
    }
    bnf.set_start(sum);
    bnf
}

#[macro_export]
macro_rules! precedenced_arith_rhs_elem {
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
    ('(') => {
        4
    };
    (')') => {
        5
    };
    ('0') => {
        6
    };
    ('1') => {
        7
    };
    ('2') => {
        8
    };
    ('3') => {
        9
    };
    ('4') => {
        10
    };
    ('5') => {
        11
    };
    ('6') => {
        12
    };
    ('7') => {
        13
    };
    ('8') => {
        14
    };
    ('9') => {
        15
    };
    ($e:expr) => {
        $e
    };
}

#[macro_export]
macro_rules! precedenced_arith {
    ($($e:tt)+) => (
        &[$(precedenced_arith_rhs_elem!($e) + 4,)+]
    )
}
