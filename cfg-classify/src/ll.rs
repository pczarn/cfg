//! The Ll grammar class.

use std::collections::BTreeMap;

use cfg_grammar::symbol_bit_set::SymbolBitSet;
use cfg_grammar::Cfg;
use cfg_predict_sets::{CfgSetsExt, PredictSets};
use cfg_symbol::Symbol;

/// LL parse table.
pub struct LlParseTable<'a> {
    map: BTreeMap<LlParseTableKey, Vec<usize>>,
    grammar: &'a Cfg,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct LlParseTableKey {
    nonterminal: Symbol,
    terminal: Symbol,
}

/// Container for classifying nonterminals as LL(1) or context-free.
#[derive(Debug, Eq, PartialEq)]
pub struct LlClassification {
    classes: BTreeMap<Symbol, LlNonterminalClass>,
}

/// A nonterminal class.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LlNonterminalClass {
    /// LL(1) class.
    Ll1,
    /// Context-free class.
    ContextFree,
}

impl<'a> LlParseTable<'a> {
    /// Creates an LL parse table.
    pub fn new(grammar: &'a Cfg) -> Self {
        let mut this = LlParseTable {
            map: BTreeMap::new(),
            grammar,
        };
        let first = grammar.first_sets();
        let follow = grammar.follow_sets_with_first(&first);
        // LlParseTable[A,a] contains the rule A → w if and only if
        // a is in FIRST(w) or
        // None is in FIRST(w) and a is in FOLLOW(A).
        for (rule_idx, rule) in grammar.rules().enumerate() {
            let rhs_first_set = first.first_set_for_string(&rule.rhs[..]);
            for &terminal in &rhs_first_set[..] {
                let key = LlParseTableKey {
                    nonterminal: rule.lhs,
                    terminal,
                };
                let entry = this.map.entry(key).or_default();
                entry.push(rule_idx);
            }
            if rhs_first_set.has_none() {
                let lhs_follow_set = follow.predict_sets().get(&rule.lhs).unwrap();
                for &terminal in &lhs_follow_set[..] {
                    let key = LlParseTableKey {
                        nonterminal: rule.lhs,
                        terminal,
                    };
                    let entry = this.map.entry(key).or_default();
                    entry.push(rule_idx);
                }
            }
        }
        this
    }

    /// Classifies nonterminals as LL(1) or context-free.
    pub fn classify(&self) -> LlClassification {
        let mut result = LlClassification {
            classes: BTreeMap::new(),
        };
        for (key, rules) in &self.map {
            if rules.len() > 1 {
                result
                    .classes
                    .insert(key.nonterminal, LlNonterminalClass::ContextFree);
            } else {
                result
                    .classes
                    .entry(key.nonterminal)
                    .or_insert(LlNonterminalClass::Ll1);
            }
        }
        // TODO: missing code here?
        let mut property = SymbolBitSet::from_elem(self.grammar, false);
        for (&nonterminal, &class) in result.classes.iter() {
            if let LlNonterminalClass::ContextFree = class {
                property.set(nonterminal, true);
            }
        }
        self.grammar.rhs_closure_for_any(&mut property);
        for (&nonterminal, class) in result.classes.iter_mut() {
            if property[nonterminal] {
                *class = LlNonterminalClass::ContextFree;
            }
        }
        result
    }
}

impl LlClassification {
    /// Access classes.
    pub fn classes(&self) -> &BTreeMap<Symbol, LlNonterminalClass> {
        &self.classes
    }
}
