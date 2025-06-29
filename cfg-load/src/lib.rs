#![deny(unsafe_code)]

use cfg_history::RootHistoryNode;
use tiny_earley::{grammar, forest, Recognizer, Symbol};

use cfg_grammar::Cfg;
use cfg_sequence::CfgSequenceExt;
use std::{collections::HashMap, convert::AsRef, fmt::{self, Write}, str::Chars};

use elsa::FrozenIndexSet;

pub struct StringInterner {
    set: FrozenIndexSet<String>,
}

impl StringInterner {
    pub fn new() -> Self {
        StringInterner {
            set: FrozenIndexSet::new(),
        }
    }

    pub fn get_or_intern<T>(&self, value: T) -> usize
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
    Ident(String),
    Rules(Vec<Rule>),
    Rhs(Vec<Vec<Fragment>>),
    Fragment(Fragment),
    Alt(Vec<Fragment>),
    None,
}

struct Evaluator {
    symbols: [Symbol; 11],
    tokens: Vec<Token>,
}

impl forest::Eval for Evaluator {
    type Elem = Value;

    fn leaf(&self, terminal: Symbol, values: u32) -> Self::Elem {
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment] =
            self.symbols;
        if terminal == ident {
            self.tokens[values as usize].ident()
        } else {
            Value::None
        }
    }

    fn product(&self, action: u32, args: Vec<Self::Elem>) -> Self::Elem {
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment] =
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
            // rule ::= lhs bnf_op semicolon;
            (4, Value::Ident(lhs), ..) => {
                let rules = vec![Rule { lhs: lhs.clone(), rhs: vec![] }];
                Value::Rules(rules)
            }
            // rule ::= lhs bnf_op rhs semicolon;
            (5, Value::Ident(lhs), _, Value::Rhs(rhs)) => {
                let rules = rhs.into_iter().map(|rhs| Rule { lhs: lhs.clone(), rhs }).collect();
                Value::Rules(rules)
            }
            // rhs ::= rhs pipe alt;
            (6, Value::Rhs(mut rhs), _, Value::Alt(alt)) => {
                rhs.push(alt);
                Value::Rhs(rhs)
            }
            // rhs ::= alt;
            (7, Value::Alt(alt), _, _) => {
                Value::Rhs(vec![alt])
            }
            // alt ::= alt fragment;
            (8, Value::Alt(mut alt), Value::Fragment(fragment), _) => {
                alt.push(fragment);
                Value::Alt(alt)
            }
            // alt ::= fragment;
            (9, Value::Fragment(fragment), _, _) => {
                Value::Alt(vec![fragment])
            }
            // fragment ::= ident op_plus;
            (10, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment { ident, rep: Rep::OneOrMore })
            }
            // fragment ::= ident op_mul;
            (11, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment { ident, rep: Rep::ZeroOrMore })
            }
            // fragment ::= ident;
            (12, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment { ident, rep: Rep::None })
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

struct Lexer<'a> {
    chars: Chars<'a>,
    line_no: usize,
    col_no: usize,
}

#[derive(Debug, Eq, PartialEq)]
enum Token {
    Ident(String),
    BnfOp,
    Semicolon,
    Pipe,
    Plus,
    Mul,
    Whitespace,
    Error(usize, usize),
}

impl Token {
    fn ident(&self) -> Value {
        if let &Token::Ident(ref s) = self {
            Value::Ident(s.clone())
        } else {
            panic!("cannot extract ident from token")
        }
    }
}

impl<'a> Lexer<'a> {
    fn tokenize(bnf: &str) -> Vec<Token> {
        let mut lexer = Lexer { chars: bnf.chars(), line_no: 1, col_no: 1 };
        let mut result = vec![];
        while let Some(token) = lexer.eat_token() {
            if token != Token::Whitespace {
                result.push(token);
            }
        }
        result
    }

    fn eat_token(&mut self) -> Option<Token> {
        // while let Some(' ' | '\n' | '\t') = self.peek() {
        //     self.advance();
        // }
        self.peek().map(|ch| self.eat(ch))
    }

    fn eat(&mut self, ch: char) -> Token {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => {
                let substring = self.chars.as_str();
                while let Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') = self.peek() {
                    self.advance();
                }
                Token::Ident(substring[..substring.len() - self.chars.as_str().len()].to_string())
            }
            ':' => {
                if self.chars.as_str().starts_with("::=") {
                    self.advance();
                    self.advance();
                    self.advance();
                    Token::BnfOp
                } else {
                    Token::Error(self.line_no, self.col_no)
                }
            }
            ';' => {
                self.advance();
                Token::Semicolon
            }
            '|' => {
                self.advance();
                Token::Pipe
            }
            '+' => {
                self.advance();
                Token::Plus
            }
            '*' => {
                self.advance();
                Token::Mul
            }
            ' ' | '\n' | '\t' => {
                self.advance();
                Token::Whitespace
            }
            _ => Token::Error(self.line_no, self.col_no)
        }
    }

    fn advance(&mut self) {
        match self.chars.next() {
            Some('\n') => {
                self.line_no += 1;
                self.col_no = 1;
            }
            Some(_) => {
                self.col_no += 1;
            }
            None => {}
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.as_str().chars().next()
    }
}

pub trait CfgLoadExt {
    fn load(bnf: &str) -> Result<Cfg, LoadError>;
    fn to_bnf(&self) -> String;
}

impl CfgLoadExt for Cfg {
    fn load(bnf: &str) -> Result<Cfg, LoadError> {
        use tiny_earley::Grammar;
        let bnf_grammar = grammar! {
            S = [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment]
            R = {
                start ::= start rule; // 2
                start ::= rule; // 3
                rule ::= ident bnf_op semicolon; // 4
                rule ::= ident bnf_op rhs semicolon; // 5
                rhs ::= rhs pipe alt; // 6
                rhs ::= alt; // 7
                alt ::= alt fragment; // 8
                alt ::= fragment; // 9
                fragment ::= ident op_plus; // 10
                fragment ::= ident op_mul; // 11
                fragment ::= ident; // 12
            }
        };
        let symbols = bnf_grammar.symbols();
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment] = bnf_grammar.symbols();
        let mut recognizer = Recognizer::new(&bnf_grammar);
        let tokens = Lexer::tokenize(bnf);
        for (i, ch) in tokens.iter().enumerate() {
            let terminal = match ch {
                Token::BnfOp => bnf_op,
                Token::Semicolon => semicolon,
                Token::Pipe => pipe,
                Token::Mul => op_mul,
                Token::Plus => op_plus,
                Token::Ident(_) => ident,
                Token::Whitespace => continue,
                &Token::Error(line_no, col_no) => return Err(LoadError::Parse { reason: "failed to tokenize".to_string(), line: line_no as u32, col: col_no as u32 }),
            };
            recognizer.scan(terminal, i as u32);
            let success = recognizer.end_earleme();
            if !success {
                return Err(LoadError::Parse { reason: "parse failed".to_string(), line: 1, col: 1 });
            }
            // assert!(success, "parse failed at character {}", i);
        }
        let finished_node = if let Some(node) = recognizer.finished_node {
            node
        } else {
            return Err(LoadError::Parse { reason: "parse failed: no result".to_string(), line: 1, col: 1 });
        };
        let result = recognizer
            .forest
            .evaluator(Evaluator { symbols, tokens })
            .evaluate(finished_node);
        if let Value::Rules(rules) = result {
            let mut cfg = Cfg::new();
            let intern = StringInterner::new();
            let mut sym_map = HashMap::new();
            let mut intern_empty = true;
            for (idx, rule) in rules.into_iter().enumerate() {
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
                cfg.rule(lhs_sym).history(RootHistoryNode::Origin { origin: idx + 1 }.into()).rhs(rhs_syms);
            }
            Ok(cfg)
        } else {
            return Err(LoadError::Eval { reason: format!("evaluation failed: Expected Value::Rules, got {:?}", result) });
        }
    }

    fn to_bnf(&self) -> String {
        let mut result = String::new();
        for rule in self.rules() {
            let mut rhs = String::new();
            for &sym in &rule.rhs[..] {
                write!(rhs, "{} ", self.sym_source().name_of(sym)).unwrap();
            }
            writeln!(result, "{} ::= {};", self.sym_source().name_of(rule.lhs), rhs.trim()).unwrap();
        }
        result
    }
}