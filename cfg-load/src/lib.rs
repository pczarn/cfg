use tiny_earley::{grammar, Grammar, Recognizer};

use cfg_grammar::Cfg;

trait CfgLoadExt {
    fn load(bnf: String) -> Cfg;
}

impl CfgLoadExt for Cfg {
    fn load(bnf: String) -> Cfg {
        let mut bnf_grammar = grammar! {
            S = [start, rule, lhs, rhs, bnf_op, ident, colon, eq_op, alpha, ident_tail, alnum, digit, pipe, op_mul, op_plus]
            R = {
                start ::= start rule;
                start ::= rule;
                rule ::= lhs bnf_op rhs;
                rhs ::= rhs ident;
                rhs ::= ident;
                bnf_op ::= colon colon eq_op;
                lhs ::= ident;
                ident ::= alpha ident_tail;
                ident ::= alpha;
                ident_tail ::= ident_tail alnum;
                ident_tail ::= alnum; 
            }
        };
        let [start, rule, lhs, rhs, bnf_op, ident, colon, eq_op, alpha, ident_tail, alnum, digit, pipe, op_mul, op_plus] = self.grammar.symbols();
        let mut recognizer = Recognizer::new(&bnf_grammar);
        for (i, ch) in bnf.chars().enumerate() {
            let terminal = match ch {
                ':' => colon,
                '=' => eq_op,
                '0'..='9' => digit,
                '|' => pipe,
                '*' => op_mul,
                '+' => op_plus,
                'a'..='z' | 'A'..='Z' => alnum,
                ' ' => continue,
                other => panic!("invalid character {}", other),
            };
            recognizer.scan(terminal, ch as u32);
            let success = recognizer.end_earleme();
            #[cfg(feature = "debug")]
            if !success {
                self.recognizer.log_earley_set_diff();
            }
            assert!(success, "parse failed at character {}", i);
        }
        let finished_node = recognizer.finished_node.expect("parse failed");
        let result = recognizer
            .forest
            .evaluator(E { symbols })
            .evaluate(finished_node);
        if let Value::Root(rules) = result {
            let mut cfg = Cfg::new();
            for rule in rules {
                cfg.rule(rule.lhs).rhs(&rule.rhs[..]);
            }
            cfg
        } else {
            panic!("evaluation failed {:?}", result)
        }

    }
}