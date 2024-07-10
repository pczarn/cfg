#![cfg(feature = "cfg-earley")]

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
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let (grammar, start, sym_map, _) = precedenced_arith::weighted_grammar();
    let binarized = grammar.binarize();

    let mut rng = SmallRng::seed_from_u64(42);
    let to_char = |s, _: &mut _| sym_map.get(&s).cloned();
    let string = binarized
        .random(start, Some(1_000_000), &mut rng, &[], to_char)
        .map(|(_syms, chars)| chars.into_iter().collect());
    // let string = syms.map(|sym_list| {
    //     sym_list
    //         .into_iter()
    //         .map(|s| .unwrap_or('X'))
    //         .collect::<String>()
    // });
    // let string = chars.into_iter().collect();
    let expected = Ok("(5/0*1/6948/92*3614-90)-8*8-(7/615)+3/1".to_string());
    assert_eq!(string, expected);
}

#[cfg(feature = "weighted-generation")]
#[test]
fn test_precedenced_arith_with_negative_lookahead() {
    use cfg::generate::weighted::NegativeRule;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let (grammar, start, sym_map, neg) = precedenced_arith::weighted_grammar();
    let binarized = grammar.binarize();

    let mut rng = SmallRng::seed_from_u64(42);
    let neg = NegativeRule {
        sym: neg,
        chars: "0",
    };
    let to_char = |sym, _: &mut _| sym_map.get(&sym).cloned();
    let string = binarized
        .random(start, Some(1_000_000), &mut rng, &[neg], to_char)
        .map(|(_syms, chars)| chars.into_iter().collect());
    // let string = syms.map(|sym_list| {
    //     sym_list
    //         .into_iter()
    //         .map(|s| sym_map.get(&s).cloned().unwrap_or('X'))
    //         .collect::<String>()
    // });
    let expected = Ok("(5/3*(8-2)/3614/990*(98)-(7/615))-3/1-7+((4179*683)/1)".to_string());
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
