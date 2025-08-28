pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use cfg_grammar::Cfg;
use cfg_sequence::CfgSequenceExt;
use cfg_symbol::Symbol;
use regex_syntax::hir::{Class, Hir, HirKind};
use regex_syntax::Parser;

pub trait CfgRegexpExt: Sized {
    fn from_regexp(regexp: &str) -> Result<(Self, LexerMap), regex_syntax::Error>;
}

impl CfgRegexpExt for Cfg {
    fn from_regexp(regexp: &str) -> Result<(Self, LexerMap), regex_syntax::Error> {
        Parser::new().parse(regexp).map(Translator::cfg_from_hir)
    }   
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone)]
pub struct LexerClasses {
    set: BTreeSet<(char, char)>,
}

impl From<u8> for LexerClasses {
    fn from(value: u8) -> Self {
        LexerClasses { set: iter::once((value as char, (value + 1) as char)).collect() }
    }
}

impl From<char> for LexerClasses {
    fn from(value: char) -> Self {
        LexerClasses { set: iter::once((value, char::from_u32(value as u32 + 1).unwrap())).collect() }
    }
}

impl From<Class> for LexerClasses {
    fn from(value: Class) -> Self {
        let set: BTreeSet<(char, char)> = match value {
            Class::Bytes(bytes) => {
                bytes.ranges().iter().map(|range| (range.start() as char, range.end() as char)).collect()
            }
            Class::Unicode(unicode) => {
                unicode.ranges().iter().map(|range| (range.start(), range.end())).collect()
            }
        };
        LexerClasses { set }
    }
}

impl LexerClasses {
    pub fn iter(&self) -> impl Iterator<Item = (char, char)> + use<'_> {
        self.set.iter().copied()
    }
}

struct Translator {
    cfg: Cfg,
    class_map: LexerMap,
}

#[derive(Debug, Clone)]
pub struct LexerMap {
    pub classes: BTreeMap<LexerClasses, Symbol>,
    ascii: Vec<Vec<Symbol>>,
    ranges: BTreeMap<char, Vec<Symbol>>,
}

impl LexerMap {
    pub fn new() -> Self {
        LexerMap {
            classes: BTreeMap::new(),
            ascii: vec![],
            ranges: BTreeMap::new(),
        }
    }

    pub fn compute(&mut self) {
        let mut result = vec![vec![]; 256];
        for (lexer_classes, &symbol) in &self.classes {
            for &class in &lexer_classes.set {
                if class.0.is_ascii() {
                    for ascii in class.0 as u32 ..= (class.1 as u32).min(256) {
                        result[ascii as usize].push(symbol);
                    }
                }
            }
        }
        self.ascii = result;
        let mut ranges = BTreeMap::new();
        for (lexer_classes, &symbol) in &self.classes {
            for &class in &lexer_classes.set {
                ranges.entry(class.0).or_insert(vec![]).push((true, symbol));
                ranges.entry(char::from_u32(class.1 as u32 + 1).unwrap()).or_insert(vec![]).push((false, symbol));
            }
        }
        let mut result = BTreeMap::new();
        let mut work = BTreeSet::new();
        for (ch, changes) in ranges {
            for (is_added, symbol) in changes {
                if is_added {
                    work.insert(symbol);
                } else {
                    work.remove(&symbol);
                }
            }
            result.entry(ch).or_insert(vec![]).extend(work.iter().copied());
        }
        self.ranges = result;
    }

    pub fn get(&self, ch: char) -> &[Symbol] {
        if ch.is_ascii() {
            &self.ascii[ch as usize][..]
        } else {
            self.ranges.range(..=ch).next_back().map(|(_, v)| &v[..]).unwrap_or(&[])
        }
    }
}

impl Translator {
    fn cfg_from_hir(hir: Hir) -> (Cfg, LexerMap) {
        let cfg = Cfg::new();
        let class_map = LexerMap::new();
        let mut this = Self { cfg, class_map };
        let x = this.walk_hir(&hir, 0);
        let lhs = match (x.len(), x.get(0).map_or(0, |y| y.len())) {
            (0, _) => {
                let [nulling] = this.cfg.sym();
                this.cfg.rule(nulling).rhs([]);
                nulling
            }
            (1, 1) => {
                x[0][0]
            }
            (1, _) => {
                let [inner] = this.cfg.sym();
                this.cfg.rule(inner).rhs(&x[0][..]);
                inner
            }
            _ => {
                let [inner] = this.cfg.sym();
                for rule in x {
                    this.cfg.rule(inner).rhs(&*rule);
                }
                inner
            }
        };
        this.cfg.set_roots([lhs]);
        (this.cfg, this.class_map)
    }

    fn walk_hir(&mut self, hir: &Hir, depth: usize) -> Vec<Vec<Symbol>> {
        let indent = "  ".repeat(depth);

        match hir.kind() {
            HirKind::Literal(lit) => {
                let mut syms = vec![];
                for &byte in &lit.0 {
                    syms.push(*self.class_map.classes.entry(byte.into()).or_insert_with(|| self.cfg.next_sym(Some(format!("__byte{}", byte).into()))));
                }
                println!("{indent}Literal: {:?}", lit);
                vec![syms]
            }
            HirKind::Class(class) => {
                let sym = *self.class_map.classes.entry(class.clone().into()).or_insert_with(|| self.cfg.next_sym(None));
                println!("{indent}Class: {:?}", class);
                vec![vec![sym]]
            }
            HirKind::Repetition(rep) => {
                println!("{indent}Repetition: {:?}", (rep.min, rep.max, rep.greedy));
                let [lhs] = self.cfg.sym();
                let x = self.walk_hir(&rep.sub, depth + 1);
                let rhs = match (x.len(), x.get(0).map_or(0, |y| y.len())) {
                    (0, _) => {
                        unreachable!()
                    }
                    (1, 1) => {
                        x[0][0]
                    }
                    (1, _) => {
                        let [inner] = self.cfg.sym();
                        self.cfg.rule(inner).rhs(&x[0][..]);
                        inner
                    }
                    _ => {
                        let [inner] = self.cfg.sym();
                        for rule in x {
                            self.cfg.rule(inner).rhs(&*rule);
                        }
                        inner
                    }
                };
                self.cfg.sequence(lhs).inclusive(rep.min, rep.max).rhs(rhs);
                vec![vec![lhs]]
            }
            HirKind::Capture(group) => {
                println!("{indent}Group (capturing={}):", group.name.is_some());
                self.walk_hir(&group.sub, depth + 1)
            }
            HirKind::Concat(exprs) => {
                println!("{indent}Concat:");
                let mut result = vec![];
                for expr in exprs {
                    let x = self.walk_hir(expr, depth + 1);
                    match x.len() {
                        0 => {}
                        1 => {
                            result.extend(x.into_iter().next().unwrap());
                        }
                        _ => {
                            let [lhs] = self.cfg.sym();
                            for rule in x {
                                self.cfg.rule(lhs).rhs(&*rule);
                            }
                            result.push(lhs);
                        }
                    }
                }
                vec![result]
            }
            HirKind::Alternation(exprs) => {
                println!("{indent}Alternation:");
                let mut alternatives = vec![];
                for expr in exprs {
                    alternatives.extend(self.walk_hir(expr, depth + 1));
                }
                alternatives
            }
            HirKind::Look(_look) => {
                unimplemented!()
            }
            HirKind::Empty => {
                println!("{indent}Empty");
                vec![vec![]]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);


        let pattern = r"(?i)(foo|bar)\d+";
        let (result, mut map) = Cfg::from_regexp(pattern).unwrap();
        assert_eq!(result.rules().count(), 5);
        map.compute();
        assert_eq!(map.get('b').len(), 1);
        assert_eq!(map.get('c').len(), 0);
        assert_eq!(map.get('B').len(), 1);
        assert_eq!(map.get('D').len(), 0);
        assert_eq!(map.get('o').len(), 1);
        assert_eq!(map.get('🯰').len(), 1);
        assert_eq!(map.get('🯹').len(), 1);
    }
}
