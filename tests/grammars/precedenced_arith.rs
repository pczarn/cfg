use cfg::prelude::*;
use cfg::earley::Grammar;
use cfg::history::LinkedHistoryNode;

use std::collections::BTreeMap;

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
pub fn weighted_grammar() -> (Cfg, Symbol, BTreeMap<Symbol, char>, Symbol) {
    let mut bnf = Cfg::new();
    let mut map = BTreeMap::new();
    let (sum, product, factor, number, plus, minus, mul, div, lparen, rparen, neg) = bnf.sym();
    let weight = |w| LinkedHistoryNode::Weight { weight: w };
    map.insert(plus, '+');
    map.insert(minus, '-');
    map.insert(mul, '*');
    map.insert(div, '/');
    map.insert(lparen, '(');
    map.insert(rparen, ')');
    bnf
        .rule(sum)
            .rhs_with_linked_history([sum, plus, product], weight(1.0))
            .rhs_with_linked_history([sum, minus, product], weight(1.0))
            .rhs_with_linked_history([product], weight(3.0))
        .rule(product)
            .rhs_with_linked_history([product, mul, factor], weight(1.0))
            .rhs_with_linked_history([product, div, factor], weight(1.0))
            .rhs_with_linked_history([factor], weight(3.0))
        .rule(factor)
            .rhs_with_linked_history([lparen, sum, rparen], weight(1.0))
            .rhs_with_linked_history([neg, number], weight(3.0))
        .rule(neg)
            .rhs_with_linked_history([], weight(1.0));
    for ch in '0'..='9' {
        let sym = bnf.sym();
        bnf.rule(number).rhs(&[sym, number]).rhs(&[sym]);
        map.insert(sym, ch);
    }
    (bnf, sum, map, neg)
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
