//! The Ll grammar class.

use std::collections::BTreeMap;

use analysis::RhsClosure;
use prediction::{FirstSetsCollector, FollowSets};
use rule::GrammarRule;
use symbol::{Symbol, SymbolBitSet};
use ContextFree;
use ContextFreeRef;

/// LL parse table.
pub struct LlParseTable<'a, G> {
    map: BTreeMap<LlParseTableKey, Vec<usize>>,
    grammar: &'a G,
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

impl<'a, G> LlParseTable<'a, G>
where
    G: ContextFree,
    &'a G: ContextFreeRef<'a, Target = G>,
{
    /// Creates an LL parse table.
    pub fn new(grammar: &'a G, start_sym: Symbol) -> Self {
        let mut this = LlParseTable {
            map: BTreeMap::new(),
            grammar,
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
                result
                    .classes
                    .insert(key.nonterminal, LlNonterminalClass::ContextFree);
            } else {
                if !result.classes.contains_key(&key.nonterminal) {
                    result
                        .classes
                        .insert(key.nonterminal, LlNonterminalClass::Ll1);
                }
            }
        }
        let mut closure = RhsClosure::new(self.grammar);
        let mut property = SymbolBitSet::new(self.grammar, false).into_bit_vec();
        for (&nonterminal, &class) in result.classes.iter() {
            if let LlNonterminalClass::ContextFree = class {
                property.set(nonterminal.into(), true);
            }
        }
        closure.rhs_closure_for_any(&mut property);
        for (&nonterminal, class) in result.classes.iter_mut() {
            if property[nonterminal.into()] {
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
