use tiny_earley::{grammar, forest, Recognizer, Symbol};

use cfg_grammar::Cfg;
use cfg_sequence::CfgSequenceExt;
use std::{collections::HashMap, convert::AsRef, fmt};

use elsa::FrozenIndexSet;

struct StringInterner {
    set: FrozenIndexSet<String>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            set: FrozenIndexSet::new(),
        }
    }

    fn get_or_intern<T>(&self, value: T) -> usize
    where
        T: AsRef<str>,
    {
        // TODO use Entry in case the standard Entry API gets improved
        // (here to avoid premature allocation or double lookup)
        self.set.insert_full(value.as_ref().to_string()).0
    }

    // fn get<T>(&self, value: T) -> Option<usize>
    // where
    //     T: AsRef<str>,
    // {
    //     self.set.get_full(value.as_ref()).map(|(i, _r)| i)
    // }

    // fn resolve(&self, index: usize) -> Option<&str> {
    //     self.set.get_index(index)
    // }
}

#[derive(Clone, Debug)]
struct Rule {
    lhs: String,
    rhs: Vec<Fragment>,
}

#[derive(Clone, Debug)]
struct Fragment {
    ident: String,
    rep: Rep,
}

#[derive(Clone, Debug)]
enum Rep {
    None,
    ZeroOrMore,
    OneOrMore,
}

#[derive(Clone, Debug)]
enum Value {
    Digit(char),
    Alpha(char),
    Alnum(char),
    Ident(String),
    Rules(Vec<Rule>),
    Rhs(Vec<Vec<Fragment>>),
    Fragment(Fragment),
    Alt(Vec<Fragment>),
    None,
}

struct Evaluator {
    symbols: [Symbol; 17],
}

impl forest::Eval for Evaluator {
    type Elem = Value;

    fn leaf(&self, terminal: Symbol, values: u32) -> Self::Elem {
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, colon, eq_op, alpha, ident_tail, alnum, digit, pipe, op_mul, op_plus, semicolon, fragment] =
            self.symbols;
        if terminal == digit {
            Value::Digit(values as u8 as char)
        } else if terminal == alpha {
            Value::Alpha(char::from_u32(values).unwrap())
        } else {
            Value::None
        }
    }

    fn product(&self, action: u32, args: Vec<Self::Elem>) -> Self::Elem {
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, colon, eq_op, alpha, ident_tail, alnum, digit, pipe, op_mul, op_plus, semicolon, fragment] =
            self.symbols;
        // let mut iter = args.into_iter();
        match (
            action,
            args.get(0).cloned().unwrap_or(Value::None),
            args.get(1).cloned().unwrap_or(Value::None),
            args.get(2).cloned().unwrap_or(Value::None),
        ) {
            // start ::= start rule;
            (2, Value::Rules(mut rules), Value::Rules(rule), _) => {
                rules.extend(rule);
                Value::Rules(rules)
            }
            // start ::= rule;
            (3, Value::Rules(rule), _, _) => {
                Value::Rules(rule)
            }
            // rule ::= lhs bnf_op rhs semicolon;
            (4, Value::Ident(lhs), _, Value::Rhs(rhs)) => {
                let rules = rhs.into_iter().map(|rhs| Rule { lhs: lhs.clone(), rhs }).collect();
                Value::Rules(rules)
            }
            // rhs ::= rhs pipe alt;
            (5, Value::Rhs(mut rhs), _, Value::Alt(alt)) => {
                rhs.push(alt);
                Value::Rhs(rhs)
            }
            // rhs ::= alt;
            (6, Value::Alt(alt), _, _) => {
                Value::Rhs(vec![alt])
            }
            // alt ::= alt fragment;
            (7, Value::Alt(mut alt), Value::Fragment(fragment), _) => {
                alt.push(fragment);
                Value::Alt(alt)
            }
            // alt ::= fragment;
            (8, Value::Fragment(fragment), _, _) => {
                Value::Alt(vec![fragment])
            }
            // fragment ::= ident op_plus;
            (9, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment { ident, rep: Rep::OneOrMore })
            }
            // fragment ::= ident op_mul;
            (10, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment { ident, rep: Rep::ZeroOrMore })
            }
            // fragment ::= ident;
            (11, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment { ident, rep: Rep::None })
            }
            // bnf_op ::= colon colon eq_op;
            (12, _, _, _) => {
                Value::None
            }
            // ident ::= alpha ident_tail;
            (13, Value::Alpha(alpha), Value::Ident(ident), _) => {
                let mut result = String::new();
                result.push(alpha);
                result.push_str(&ident[..]);
                Value::Ident(result)
            }
            // ident ::= alpha;
            (14, Value::Alpha(ch), _, _) => {
                Value::Ident(ch.into())
            }
            // ident_tail ::= ident_tail alnum;
            (15, Value::Ident(mut ident), Value::Alnum(ch), _) => {
                ident.push(ch);
                Value::Ident(ident)
            }
            // ident_tail ::= alnum;
            (16, Value::Alnum(ch), _, _) => {
                Value::Ident(ch.into())
            }
            // alnum ::= alpha;
            (17, Value::Alpha(ch), _, _) => {
                Value::Alnum(ch)
            }
            // alnum ::= digit;
            (18, Value::Digit(digit), _, _) => {
                Value::Digit(digit)
            }
            args => panic!("unknown rule id {:?} or args {:?}", action, args),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoadError {
    Parse {
        reason: String,
        line: u32,
        col: u32,
    },
    Eval {
        reason: String,
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError::Parse { reason, line, col } => {
                write!(f, "Parse error at line {} column {}: reason: {}", line, col, reason)
            }
            LoadError::Eval { reason } => {
                write!(f, "Eval error. Reason: {}", reason)
            }
        }
    }
}

pub trait CfgLoadExt {
    fn load(bnf: &str) -> Result<Cfg, LoadError>;
}

impl CfgLoadExt for Cfg {
    fn load(bnf: &str) -> Result<Cfg, LoadError> {
        let bnf_grammar = grammar! {
            S = [start, rule, alt, rhs, bnf_op, ident, colon, eq_op, alpha, ident_tail, alnum, digit, pipe, op_mul, op_plus, semicolon, fragment]
            R = {
                start ::= start rule;
                start ::= rule;
                rule ::= ident bnf_op rhs semicolon;
                rhs ::= rhs pipe alt;
                rhs ::= alt;
                alt ::= alt fragment;
                alt ::= fragment;
                fragment ::= ident op_plus;
                fragment ::= ident op_mul;
                fragment ::= ident;
                bnf_op ::= colon colon eq_op;
                ident ::= alpha ident_tail;
                ident ::= alpha;
                ident_tail ::= ident_tail alnum;
                ident_tail ::= alnum;
                alnum ::= alpha;
                alnum ::= digit;
            }
        };
        let symbols = bnf_grammar.symbols();
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, colon, eq_op, alpha, ident_tail, alnum, digit, pipe, op_mul, op_plus, semicolon, fragment] = bnf_grammar.symbols();
        let mut recognizer = Recognizer::new(&bnf_grammar);
        let mut line_no = 1;
        let mut col_no = 1;
        for ch in bnf.chars() {
            let terminal = match ch {
                ':' => colon,
                ';' => semicolon,
                '=' => eq_op,
                '0'..='9' => digit,
                '|' => pipe,
                '*' => op_mul,
                '+' => op_plus,
                'a'..='z' | 'A'..='Z' => alpha,
                ' ' => continue,
                '\n' => {
                    line_no += 1;
                    col_no = 1;
                    continue;
                }
                other => return Err(LoadError::Parse { reason: format!("invalid character {}", other), line: line_no, col: col_no }),
            };
            recognizer.scan(terminal, ch as u32);
            let success = recognizer.end_earleme();
            // if !success {
            //     self.recognizer.log_earley_set_diff();
            // }
            if !success {
                return Err(LoadError::Parse { reason: "parse failed".to_string(), line: line_no, col: col_no });
            }
            col_no += 1;
            // assert!(success, "parse failed at character {}", i);
        }
        let finished_node = if let Some(node) = recognizer.finished_node {
            node
        } else {
            return Err(LoadError::Parse { reason: "parse failed".to_string(), line: line_no, col: col_no });
        };
        let result = recognizer
            .forest
            .evaluator(Evaluator { symbols })
            .evaluate(finished_node);
        if let Value::Rules(rules) = result {
            let mut cfg = Cfg::new();
            let intern = StringInterner::new();
            let mut sym_map = HashMap::new();
            let mut intern_empty = true;
            for rule in rules {
                let lhs = intern.get_or_intern(&rule.lhs[..]);
                let lhs_sym = *sym_map.entry(lhs).or_insert_with(|| cfg.sym_source_mut().next_sym(Some(rule.lhs[..].into())));
                if intern_empty {
                    cfg.set_roots([lhs_sym]);
                    intern_empty = false;
                }
                let rhs_syms: Vec<_> = rule.rhs.into_iter().map(|fragment| {
                    let id = intern.get_or_intern(&fragment.ident[..]);
                    let rhs_sym = *sym_map.entry(id).or_insert_with(|| cfg.sym_source_mut().next_sym(Some(fragment.ident[..].into())));
                    match fragment.rep {
                        Rep::None => rhs_sym,
                        Rep::ZeroOrMore => {
                            let [new_sym] = cfg.sym();
                            cfg.sequence(new_sym).inclusive(0, None).rhs(rhs_sym);
                            new_sym
                        }
                        Rep::OneOrMore => {
                            let [new_sym] = cfg.sym();
                            cfg.sequence(new_sym).range(1..).rhs(rhs_sym);
                            new_sym
                        }
                    }
                }).collect();
                cfg.rule(lhs_sym).rhs(rhs_syms);
            }
            Ok(cfg)
        } else {
            return Err(LoadError::Eval { reason: format!("evaluation failed: Expected Value::Rules, got {:?}", result) });
        }
    }
}