//! The Ll grammar class.

use std::collections::BTreeMap;

use ContextFree;
use ContextFreeRef;
use symbol::Symbol;
use rule::GrammarRule;
use prediction::{FirstSetsCollector, FollowSets};

/// LL parse table.
pub struct LlParseTable {
    map: BTreeMap<LlParseTableKey, Vec<usize>>,
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

impl LlParseTable {
    /// Creates an LL parse table.
    pub fn new<'a, G>(grammar: &'a G, start_sym: Symbol) -> Self
        where G: ContextFree,
            &'a G: ContextFreeRef<'a, Target = G>,
    {
        let mut this = LlParseTable {
            map: BTreeMap::new(),
        };
        let first = FirstSetsCollector::new(grammar);
        let follow = FollowSets::new(grammar, start_sym, first.first_sets());
        // LlParseTable[A,a] contains the rule A → w if and only if
        // a is in FIRST(w) or
        // None is in FIRST(w) and a is in FOLLOW(A).
        for (rule_idx, rule) in grammar.rules().enumerate() {
            let rhs_first_set = first.first_set_for_string(rule.rhs());
            let identity = |maybe_terminal| maybe_terminal;
            for &terminal in rhs_first_set.iter().flat_map(identity) {
                let key = LlParseTableKey {
                    nonterminal: rule.lhs(),
                    terminal,
                };
                let entry = this.map.entry(key).or_insert(vec![]);
                entry.push(rule_idx);
            }
            if rhs_first_set.contains(&None) {
                let lhs_follow_set = follow.follow_sets().get(&rule.lhs()).unwrap();
                for &terminal in lhs_follow_set.iter().flat_map(identity) {
                    let key = LlParseTableKey {
                        nonterminal: rule.lhs(),
                        terminal,
                    };
                    let entry = this.map.entry(key).or_insert(vec![]);
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
        for (key, ref rules) in &self.map {
            if rules.len() > 1 {
                result.classes.insert(key.nonterminal, LlNonterminalClass::ContextFree);
            } else {
                if !result.classes.contains_key(&key.nonterminal) {
                    result.classes.insert(key.nonterminal, LlNonterminalClass::Ll1);
                }
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
