#![deny(unsafe_code)]

use cfg_history::RootHistoryNode;
use tiny_earley::{grammar, forest, Recognizer, Symbol};

use cfg_grammar::{Cfg, SymbolBitSet};
use cfg_sequence::CfgSequenceExt;
use std::{collections::{HashMap, HashSet}, convert::AsRef, fmt::{self, Write}, str::Chars};

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

    fn get<T>(&self, value: T) -> Option<usize>
    where
        T: AsRef<str>,
    {
        self.set.get_full(value.as_ref()).map(|(i, _r)| i)
    }

    // fn resolve(&self, index: usize) -> Option<&str> {
    //     self.set.get_index(index)
    // }
}

#[derive(Clone, Debug)]
struct Rule {
    lhs: String,
    rhs: Vec<Fragment>,
    action: Option<String>,
}

#[derive(Clone, Debug)]
enum Fragment {
    Rhs {
        ident: String,
        rep: Rep,
    },
    Lex {
        string: String,
    },
    Call {
        func: String,
        arg: Box<Fragment>,
    },
}

#[derive(Clone, Copy, Debug)]
enum Rep {
    None,
    ZeroOrMore,
    OneOrMore,
}

#[derive(Clone, Debug)]
enum Value {
    Ident(String),
    String(String),
    Start(Vec<Rule>, Option<LexerVal>),
    Rules(Vec<Rule>),
    Lex(LexerVal),
    Rhs(Vec<(Vec<Fragment>, Option<String>)>),
    Fragment(Fragment),
    Alt(Vec<Fragment>),
    Alt2(Vec<Fragment>, Option<String>),
    None,
}

#[derive(Clone, Debug)]
struct LexerVal(Vec<Rule>);

struct Evaluator {
    symbols: [Symbol; 23],
    tokens: Vec<(Token, usize, usize)>,
}

impl forest::Eval for Evaluator {
    type Elem = Value;

    fn leaf(&self, terminal: Symbol, values: u32) -> Self::Elem {
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment, string, decl, action, lexer_keyword, lexer, lbrace, rbrace, rules, gt_op, lparen, rparen, alt2] =
            self.symbols;
        if terminal == ident {
            self.tokens[values as usize].0.ident()
        } else if terminal == string {
            self.tokens[values as usize].0.string()
        } else {
            Value::None
        }
    }

    fn product(&self, action_num: u32, args: Vec<Self::Elem>) -> Self::Elem {
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment, string, decl, action, lexer_keyword, lexer, lbrace, rbrace, rules, gt_op, lparen, rparen, alt2] =
            self.symbols;
        // let mut iter = args.into_iter();
        match (
            action_num,
            args.get(0).cloned().unwrap_or(Value::None),
            args.get(1).cloned().unwrap_or(Value::None),
            args.get(2).cloned().unwrap_or(Value::None),
        ) {
            // start ::= start decl;
            (2, Value::Start(mut rules, lex), Value::Rules(rule), _) => {
                rules.extend(rule);
                Value::Start(rules, lex)
            }
            // start ::= start decl;
            (2, Value::Start(mut rules, None), Value::Lex(lex), _) => {
                Value::Start(rules, Some(lex))
            }
            // start ::= decl;
            (3, Value::Rules(rule), _, _) => {
                Value::Start(rule, None)
            }
            // decl ::= rule;
            (4, Value::Rules(rules), _, _) => {
                Value::Rules(rules)
            }
            // decl ::= lexer;
            (5, Value::Lex(lex), _, _) => {
                Value::Lex(lex)
            }
            // rule ::= lhs bnf_op semicolon;
            (6, Value::Ident(lhs), ..) => {
                let rules = vec![Rule { lhs: lhs.clone(), rhs: vec![], action: None }];
                Value::Rules(rules)
            }
            // rule ::= lhs bnf_op action semicolon;
            (7, Value::Ident(lhs), _, Value::Ident(action)) => {
                let rules = vec![Rule { lhs: lhs.clone(), rhs: vec![], action: Some(action) }];
                Value::Rules(rules)
            }
            // rule ::= lhs bnf_op rhs semicolon;
            (8, Value::Ident(lhs), _, Value::Rhs(rhs)) => {
                let rules = rhs.into_iter().map(|(rhs, action)| Rule { lhs: lhs.clone(), rhs, action }).collect();
                Value::Rules(rules)
            }
            // rhs ::= rhs pipe alt2;
            (9, Value::Rhs(mut rhs), _, Value::Alt2(alt, action)) => {
                rhs.push((alt, action));
                Value::Rhs(rhs)
            }
            // rhs ::= alt2;
            (10, Value::Alt2(alt, action), _, _) => {
                Value::Rhs(vec![(alt, action)])
            }
            // alt2 ::= alt;
            (11, Value::Alt(alt), _, _) => {
                Value::Alt2(alt, None)
            }
            // alt2 ::= alt action;
            (12, Value::Alt(alt), Value::Ident(action), _) => {
                Value::Alt2(alt, Some(action))
            }
            // alt ::= alt fragment;
            (13, Value::Alt(mut alt), Value::Fragment(fragment), _) => {
                alt.push(fragment);
                Value::Alt(alt)
            }
            // alt ::= fragment;
            (14, Value::Fragment(fragment), _, _) => {
                Value::Alt(vec![fragment])
            }
            // fragment ::= ident op_plus;
            (15, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment::Rhs { ident, rep: Rep::OneOrMore })
            }
            // fragment ::= ident op_mul;
            (16, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment::Rhs { ident, rep: Rep::ZeroOrMore })
            }
            // fragment ::= ident;
            (17, Value::Ident(ident), _, _) => {
                Value::Fragment(Fragment::Rhs { ident, rep: Rep::None })
            }
            // fragment ::= string;
            (18, Value::String(string), _, _) => {
                Value::Fragment(Fragment::Lex { string })
            }
            // fragment ::= ident rparen string rparen;
            (19, Value::Ident(ident), _, Value::String(string)) => {
                Value::Fragment(Fragment::Call { func: ident, arg: Box::new(Fragment::Lex { string }) })
            }
            // fragment ::= ident rparen string rparen;
            (20, _, _, Value::Rules(rules)) => {
                Value::Lex(LexerVal(rules))
            }
            // action ::= gt_op ident;
            (21, _, Value::Ident(name), _) => {
                Value::Ident(name)
            }
            // rules ::= rule;
            (22, Value::Rules(rules), _, _) => {
                Value::Rules(rules)
            }
            // rules ::= rules rule;
            (23, Value::Rules(mut rules), Value::Rules(rules2), _) => {
                rules.extend(rules2);
                Value::Rules(rules)
            }
            args => panic!("unknown rule id {:?} or args {:?}", args.0, args),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoadError {
    Parse {
        reason: String,
        line: u32,
        col: u32,
        token: Option<Token>,
    },
    Eval {
        reason: String,
    },
    Lex {
        reason: String,
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError::Parse { reason, line, col, token } => {
                write!(f, "Parse error at line {} column {}: reason: {} token: {:?}", line, col, reason, token)
            }
            LoadError::Eval { reason } => {
                write!(f, "Eval error. Reason: {}", reason)
            }
            LoadError::Lex { reason } => {
                write!(f, "Lexical grammar error. Reason: {}", reason)
            }
        }
    }
}

struct Lexer<'a> {
    chars: Chars<'a>,
    line_no: usize,
    col_no: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Token {
    Ident(String),
    BnfOp,
    Semicolon,
    Pipe,
    Plus,
    Mul,
    Whitespace,
    String(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
    GtOp,
    LexerKeyword,
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

    fn string(&self) -> Value {
        if let &Token::String(ref s) = self {
            Value::String(s.clone())
        } else {
            panic!("cannot extract ident from token")
        }
    }
}

impl<'a> Lexer<'a> {
    fn tokenize(bnf: &str) -> Vec<(Token, usize, usize)> {
        let mut lexer = Lexer { chars: bnf.chars(), line_no: 1, col_no: 1 };
        let mut result = vec![];
        let mut line_no = 1;
        let mut col_no = 1;
        while let Some(token) = lexer.eat_token() {
            if token != Token::Whitespace {
                result.push((token, line_no, col_no));
            }
            line_no = lexer.line_no;
            col_no = lexer.col_no;
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
                let result = substring[..substring.len() - self.chars.as_str().len()].to_string();
                if result == "lexer" {
                    Token::LexerKeyword
                } else {
                    Token::Ident(result)
                }
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
            '(' => {
                self.advance();
                Token::LParen
            }
            ')' => {
                self.advance();
                Token::RParen
            }
            '{' => {
                self.advance();
                Token::LBrace
            }
            '}' => {
                self.advance();
                Token::RBrace
            }
            '>' => {
                self.advance();
                Token::GtOp
            }
            ' ' | '\n' | '\t' => {
                self.advance();
                Token::Whitespace
            }
            '"' => {
                let mut result = String::new();
                self.advance();
                while self.peek().map_or(false, |ch| ch != '"') {
                    match self.peek() {
                        None => break,
                        Some('\\') => {
                            self.advance();
                            if let Some(ch) = self.peek() {
                                result.push(ch);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        Some(other) => {
                            self.advance();
                            result.push(other);
                        }
                    }
                }
                if Some('"') == self.peek() {
                    self.advance();
                    Token::String(result)
                } else {
                    Token::Error(self.line_no, self.col_no)
                }
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

pub trait CfgLoadAdvancedExt {
    fn load_advanced(grammar: &str) -> Result<(Cfg, SymbolBitSet), LoadError>;
    fn to_bnf(&self) -> String;
}

// fn lexeme_set(cfg: &Cfg, lexeme_origin: usize) -> Result<SymbolBitSet, LoadError> {
//     let terminal_set = cfg.terminal_symbols();
//     let mut lexeme_set = SymbolBitSet::new();
//     lexeme_set.reset(cfg.sym_source());
//     for rule in cfg.rules() {
//         if rule.history.origin().id >= lexeme_origin as u32 {
//             // this is a lexical rule

//         }
//     }
// }

fn check_for_lexical_error(rules: &Vec<Rule>, lexer: Option<&LexerVal>) {
    // TODO
}

impl CfgLoadAdvancedExt for Cfg {
    fn load_advanced(grammar: &str) -> Result<(Cfg, SymbolBitSet), LoadError> {
        use tiny_earley::Grammar;
        let bnf_grammar = grammar! {
            S = [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment, string, decl, action, lexer_keyword, lexer, lbrace, rbrace, rules, gt_op, lparen, rparen, alt2]
            R = {
                start ::= start decl; // 2
                start ::= decl; // 3
                decl ::= rule; // 4
                decl ::= lexer; // 5
                rule ::= ident bnf_op semicolon; // 6
                rule ::= ident bnf_op action semicolon; // 7
                rule ::= ident bnf_op rhs semicolon; // 8
                rhs ::= rhs pipe alt2; // 9
                rhs ::= alt2; // 10
                alt2 ::= alt; // 11
                alt2 ::= alt action; // 12
                alt ::= alt fragment; // 13
                alt ::= fragment; // 14
                fragment ::= ident op_plus; // 15
                fragment ::= ident op_mul; // 16
                fragment ::= ident; // 17
                fragment ::= string; // 18
                fragment ::= ident lparen string rparen; // 19
                lexer ::= lexer_keyword lbrace rules rbrace; // 20
                action ::= gt_op ident; // 21
                rules ::= rule; // 22
                rules ::= rules rule; // 23
            }
        };
        let symbols = bnf_grammar.symbols();
        #[allow(unused_variables)]
        let [start, rule, alt, rhs, bnf_op, ident, pipe, op_mul, op_plus, semicolon, fragment, string, decl, action, lexer_keyword, lexer, lbrace, rbrace, rules, gt_op, lparen, rparen, alt2] = bnf_grammar.symbols();
        let mut recognizer = Recognizer::new(&bnf_grammar);
        let tokens = Lexer::tokenize(grammar);
        for (i, &(ref ch, line, col)) in tokens.iter().enumerate() {
            let terminal = match ch {
                Token::BnfOp => bnf_op,
                Token::Semicolon => semicolon,
                Token::Pipe => pipe,
                Token::Mul => op_mul,
                Token::Plus => op_plus,
                Token::Ident(_) => ident,
                Token::String(_) => string,
                Token::LBrace => lbrace,
                Token::RBrace => rbrace,
                Token::LParen => lparen,
                Token::RParen => rparen,
                Token::GtOp => gt_op,
                Token::LexerKeyword => lexer_keyword,
                Token::Whitespace => continue,
                &Token::Error(line_no, col_no) => return Err(LoadError::Parse { reason: "failed to tokenize".to_string(), line: line_no as u32, col: col_no as u32, token: None }),
            };
            recognizer.scan(terminal, i as u32);
            let success = recognizer.end_earleme();
            if !success {
                return Err(LoadError::Parse { reason: "parse failed".to_string(), line: line as u32, col: col as u32, token: Some(ch.clone()) });
            }
            // assert!(success, "parse failed at character {}", i);
        }
        let finished_node = if let Some(node) = recognizer.finished_node {
            node
        } else {
            return Err(LoadError::Parse { reason: "parse failed: no result".to_string(), line: 1, col: 1, token: None });
        };
        let result = recognizer
            .forest
            .evaluator(Evaluator { symbols, tokens })
            .evaluate(finished_node);
        #[derive(Hash, Eq, PartialEq, Debug)]
        enum Place { CfgLhs, CfgRhs, LexLhs, LexRhs }

        if let Value::Start(rules, lexer) = result {
            let mut set_of_places: HashSet<(String, Place)> = HashSet::new();
            let mut cfg = Cfg::new();
            let intern = StringInterner::new();
            let lex_string_intern = StringInterner::new();
            let mut sym_map = HashMap::new();
            let mut intern_empty = true;
            let max_origin = rules.len();
            check_for_lexical_error(&rules, lexer.as_ref());
            for (idx, rule) in rules.into_iter().chain(lexer.unwrap_or(LexerVal(vec![])).0.into_iter()).enumerate() {
                let lhs = intern.get_or_intern(&rule.lhs[..]);
                let lhs_sym = *sym_map.entry(lhs).or_insert_with(|| cfg.sym_source_mut().next_sym(Some(rule.lhs[..].into())));
                if intern_empty {
                    cfg.set_roots([lhs_sym]);
                    intern_empty = false;
                }
                let rhs_syms: Result<Vec<_>, LoadError> = rule.rhs.into_iter().map(|fragment| {
                    match fragment {
                        Fragment::Call { func, arg } => {
                            if func != "Regexp" {
                                return Err(LoadError::Lex { reason: format!("expected 'Regexp', found '{}'", func) });
                            }
                            match &*arg {
                                &Fragment::Call { ref func, ref arg } => {
                                    unreachable!()
                                }
                                &Fragment::Lex { ref string } => {
                                    unimplemented!()
                                }
                                &Fragment::Rhs { ref ident, rep } => {
                                    unreachable!()
                                }
                            }
                        }
                        Fragment::Lex { string } => {
                            let id = lex_string_intern.get_or_intern(&string[..]);
                            let name = format!("__lex{}", id);
                            let id = intern.get_or_intern(&name[..]);
                            let rhs_sym = *sym_map.entry(id).or_insert_with(|| {
                                cfg.sym_source_mut().next_sym(Some(name.clone().into()))
                            });
                            Ok(rhs_sym)
                        }
                        Fragment::Rhs { ident, rep } => {
                            let id = intern.get_or_intern(&ident[..]);
                            let rhs_sym = *sym_map.entry(id).or_insert_with(|| cfg.sym_source_mut().next_sym(Some(ident[..].into())));
                            match rep {
                                Rep::None => Ok(rhs_sym),
                                Rep::ZeroOrMore => {
                                    let [new_sym] = cfg.sym();
                                    cfg.sequence(new_sym).inclusive(0, None).rhs(rhs_sym);
                                    Ok(new_sym)
                                }
                                Rep::OneOrMore => {
                                    let [new_sym] = cfg.sym();
                                    cfg.sequence(new_sym).range(1..).rhs(rhs_sym);
                                    Ok(new_sym)
                                }
                            }
                        }
                    }
                }).collect();
                cfg.rule(lhs_sym).history(RootHistoryNode::Origin { origin: idx + 1 }.into()).rhs(rhs_syms?);
            }
            Ok((cfg, SymbolBitSet::new()))
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
