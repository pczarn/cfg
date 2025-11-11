#[macro_use]
mod grammars;

#[cfg(feature = "weighted-generation")]
use cfg::generate::weighted::Random;
#[cfg(feature = "weighted-generation")]
use grammars::*;

// const SUM_TOKENS: &'static [u32] = precedenced_arith!(
//     '1' '+' '(' '2' '*' '3' '-' '4' ')' '/'
//     '(' '5' '5' ')' '-' '(' '5' '4' ')' '*'
//     '5' '5' '+' '6' '2' '-' '1' '3' '-' '('
//     '(' '3' '6' ')' ')'
// );

#[cfg(feature = "weighted-generation")]
#[test]
fn test_precedenced_arith() {
    use cfg::generate::weighted::random::Limits;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let (mut grammar, start, sym_map, _) = precedenced_arith::weighted_grammar();
    grammar.limit_rhs_len(Some(2));

    let mut rng = SmallRng::seed_from_u64(4242);
    let to_char = |s, _: &mut _| sym_map.get(&s).cloned();
    let string = grammar
        .random(
            start,
            Some(Limits {
                terminals: 1_000_000,
                negative_rule_attempts: 1024,
            }),
            &mut rng,
            &[],
            to_char,
        )
        .map(|(_syms, chars)| chars.into_iter().collect());
    // let string = syms.map(|sym_list| {
    //     sym_list
    //         .into_iter()
    //         .map(|s| .unwrap_or('X'))
    //         .collect::<String>()
    // });
    // let string = chars.into_iter().collect();
    let expected = Ok("33/5*10*6+2-890/(((5/6601-40)/970-1/0-40/8*2/70-(9*91700))*(768-(6/26*(1+508/41-(97))/((((315483)*86*2/0)-8-61*0436238*72629)+8*66)+69/9667974*43-1))/3)/8*(830/(3+5/460/01/02-0066/956-(8*45*(((((4969)/5))-(3*6+4/9796416*((3*627-5*66/(77)/(316/0/84/51+8)+6+3*866/4)))+7/7+578/749)*6/(7)*88*6)+87*03/9*(8))/7412/50)+1)".to_string());
    assert_eq!(string, expected);
}

#[cfg(feature = "weighted-generation")]
#[test]
fn test_precedenced_arith_with_negative_lookahead() {
    use cfg::generate::weighted::NegativeRule;
    use cfg::generate::weighted::random::Limits;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let (mut grammar, start, sym_map, neg) = precedenced_arith::weighted_grammar();
    grammar.limit_rhs_len(Some(2));

    let mut rng = SmallRng::seed_from_u64(4242);
    let neg = NegativeRule {
        sym: neg,
        chars: "0",
    };
    let to_char = |sym, _: &mut _| sym_map.get(&sym).cloned();
    let string = grammar
        .random(
            start,
            Some(Limits {
                terminals: 1_000_000,
                negative_rule_attempts: 1024,
            }),
            &mut rng,
            &[neg],
            to_char,
        )
        .map(|(_syms, chars)| chars.into_iter().collect());
    // let string = syms.map(|sym_list| {
    //     sym_list
    //         .into_iter()
    //         .map(|s| sym_map.get(&s).cloned().unwrap_or('X'))
    //         .collect::<String>()
    // });
    let expected = Ok("33/5*10*6+2-890/(((5/6601-40)/970-1/3-40/8*2-828)*(9)/1700)/(768-(6/26*(1+508/41-(97))/((((315483)*86*2/5)-9-1)+((2629-8*66)/(9-67974/43)))+1-48))*(830/(3+5/460/1/2-66/956-(8*45*(((((4969)/5))-(3*6+4/9796416*((3*627-5*66/(77)/(316/8/4/51+8)+6+3*866/4)))+7/7+578/749)*6/(7)*88*6)+87*3/9*(8))/7412/50)+1)".to_string());
    assert_eq!(string, expected);
}

// #[test]
// fn test_ambiguous_arithmetic() {
//     let tokens = ambiguous_arith!('2' '-' '0' '*' '3' '+' '1');
//     let external = ambiguous_arith::grammar();
//     let cfg = InternalGrammar::<PerformancePolicy16>::from_grammar(&external);
//     let mut evaluator = SimpleEvaluator::new(
//         ambiguous_arith::leaf,
//         ambiguous_arith::rule,
//         |_, _: &mut Vec<i32>| unreachable!()
//     );
//     let bocage = Bocage::new(&cfg);
//     let mut rec = Recognizer::new(&cfg, bocage);
//     assert!(rec.parse(tokens));
//     let mut traverse = rec.forest.traverse();
//     let results = evaluator.traverse(&mut traverse, rec.finished_node().unwrap());

//     // The result is currently ordered by rule ID:
//     assert_eq!(results, vec![2, 1, 3, 7, 8]);

//     // A result ordered by structure would be: [2, 1, 8, 3, 7]
//     // where

//     // 2  =  2 - (0 * (3 + 1))
//     // 1  =  2 - ((0 * 3) + 1)
//     // 8  =  (2 - 0) * (3 + 1)
//     // 3  =  (2 - (0 * 3)) + 1
//     // 7  =  ((2 - 0) * 3) + 1
// }
