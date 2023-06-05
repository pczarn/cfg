extern crate cfg;
#[cfg(feature = "rand")]
extern crate rand;

#[macro_use]
mod grammars;

#[cfg(feature = "rand")]
use cfg::generate::weighted::Random;
#[cfg(feature = "generation")]
use grammars::*;

// const SUM_TOKENS: &'static [u32] = precedenced_arith!(
//     '1' '+' '(' '2' '*' '3' '-' '4' ')' '/'
//     '(' '5' '5' ')' '-' '(' '5' '4' ')' '*'
//     '5' '5' '+' '6' '2' '-' '1' '3' '-' '('
//     '(' '3' '6' ')' ')'
// );

#[cfg(feature = "generation")]
#[test]
fn test_precedenced_arith() {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let grammar = precedenced_arith::weighted_grammar();
    let binarized = grammar.binarize();

    let mut rng = SmallRng::seed_from_u64(42);
    let string = binarized.random(Some(1_000_000), &mut rng);
    let expected = Ok(vec![
        11u32.into(),
        15u32.into(),
        17u32.into(),
        7u32.into(),
        9u32.into(),
        18u32.into(),
        6u32.into(),
        10u32.into(),
        5u32.into(),
        18u32.into(),
        14u32.into(),
        19u32.into(),
        7u32.into(),
        13u32.into(),
        4u32.into(),
        10u32.into(),
        19u32.into(),
        19u32.into(),
        15u32.into(),
        6u32.into(),
        18u32.into(),
        8u32.into(),
        4u32.into(),
        9u32.into(),
        17u32.into(),
        5u32.into(),
        13u32.into(),
        6u32.into(),
        13u32.into(),
        8u32.into(),
    ]);
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
