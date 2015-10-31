//#[cfg(feature = "nightly")]
//use collections::range::RangeArgument;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::marker::PhantomData;

use history::{Action, RewriteSequence};
use rule_builder::{RuleBuilder, HistoryFn};
use rule_container::RuleContainer;
use sequence::{Separator, Sequence};
use sequence::Separator::{Trailing, Proper, Liberal};
use sequence_builder::SequenceRuleBuilder;
use symbol::{GrammarSymbol, SymbolSource};

/// Trait for storing sequence rules in containers, with potential rewrites.
pub trait SequenceDestination<H> {
    /// The type of symbols.
    type Symbol;
    /// Inserts a sequence rule.
    fn add_sequence(&mut self, seq: Sequence<H, Self::Symbol>);
}

pub struct SequencesToProductions<H, D> where
            H: RewriteSequence,
            D: RuleContainer {
    destination: D,
    stack: Vec<Sequence<H::Rewritten, D::Symbol>>,
    map: HashMap<PartialSequence<D::Symbol>, D::Symbol>,
    history: Option<H>,
    default_history: Option<H::Rewritten>,
}

// A key into a private map.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PartialSequence<S> {
    rhs: S,
    start: u32,
    end: Option<u32>,
    separator: Separator<S>,
}

impl<'a, H, S> SequenceDestination<H> for &'a mut Vec<Sequence<H, S>> where S: GrammarSymbol {
    type Symbol = S;

    fn add_sequence(&mut self, seq: Sequence<H, Self::Symbol>) {
        self.push(seq);
    }
}

impl<H, S, D> SequenceDestination<H> for SequencesToProductions<H, D> where
            D: RuleContainer<History=H::Rewritten, Symbol=S>,
            H: RewriteSequence,
            H::Rewritten: Clone,
            S: GrammarSymbol {
    type Symbol = S;

    fn add_sequence(&mut self, seq: Sequence<H, Self::Symbol>) {
        self.rewrite(seq);
    }
}

impl<H, S, D> SequencesToProductions<H, D> where
            D: RuleContainer<History=H::Rewritten, Symbol=S>,
            H: RewriteSequence,
            H::Rewritten: Clone,
            S: GrammarSymbol {
    pub fn new(destination: D) -> Self {
        SequencesToProductions {
            destination: destination,
            stack: vec![],
            map: HashMap::new(),
            history: None,
            default_history: None,
        }
    }

    pub fn rewrite_sequences(sequence_rules: &[Sequence<H, S>], rules: D) {
        let mut rewrite = SequenceRuleBuilder::new(SequencesToProductions::new(rules));
        for rule in sequence_rules {
            rewrite = rewrite.sequence(rule.lhs)
                             .separator(rule.separator)
                             .inclusive(rule.start, rule.end)
                             .rhs_with_history(rule.rhs, &rule.history);
        }
    }

    pub fn rewrite(&mut self, top: Sequence<H, S>) {
        self.stack.clear();
        self.map.clear();
        let seq_history = top.history.sequence(&top);
        self.history = Some(top.history);
        self.stack.push(Sequence {
            lhs: top.lhs,
            rhs: top.rhs,
            start: top.start,
            end: top.end,
            separator: top.separator,
            history: seq_history,
        });

        while let Some(seq) = self.stack.pop() {
            assert!(seq.start <= seq.end.unwrap_or(!0));
            self.reduce(seq);
        }
    }

    fn rule(&mut self, lhs: S) -> RuleBuilder<&mut D, CloneHistory<H::Rewritten, S>> {
        let default = CloneHistory::new(self.default_history.as_ref().unwrap());
        RuleBuilder::new(&mut self.destination).rule(lhs).default_history(default)
    }

    fn recurse(&mut self, seq: Sequence<H::Rewritten, S>) -> S {
        let sym_source = &mut self.destination;
        // As a placeholder
        let partial = PartialSequence {
            rhs: seq.rhs,
            separator: seq.separator,
            start: seq.start,
            end: seq.end
        };

        match self.map.entry(partial) {
            Entry::Vacant(vacant) => {
                let lhs = sym_source.sym();
                vacant.insert(lhs);
                self.stack.push(Sequence {
                    lhs: lhs,
                    rhs: seq.rhs,
                    start: seq.start,
                    end: seq.end,
                    separator: seq.separator,
                    history: seq.history,
                });
                lhs
            }
            Entry::Occupied(lhs) => {
                *lhs.get()
            }
        }
    }

    fn reduce(&mut self, sequence: Sequence<H::Rewritten, S>) {
        let Sequence { lhs, rhs, start, end, separator, ref history } = sequence;
        let sequence = Sequence { lhs: lhs, rhs: rhs, start: start, end: end,
            separator: separator, history: history.no_op() };
        self.default_history = Some(history.clone());

        match (separator, start, end) {
            (Liberal(sep), _, _) => {
                let sym1 = self.recurse(sequence.clone().separator(Proper(sep)));
                let sym2 = self.recurse(sequence.clone().separator(Trailing(sep)));
                // seq ::= sym1 | sym2
                self.rule(lhs).rhs([sym1])
                              .rhs([sym2]);
            }
            (Trailing(sep), _, _) => {
                let sym = self.recurse(sequence.separator(Proper(sep)));
                // seq ::= sym sep
                self.rule(lhs).rhs([sym, sep]);
            }
            (_, 0, end) => {
                // seq ::= epsilon | sym
                self.rule(lhs).rhs([]);
                if end != Some(0) {
                    let sym = self.recurse(sequence.inclusive(1, end));
                    self.rule(lhs).rhs([sym]);
                }
            }
            (separator, 1, None) => {
                // seq ::= item
                self.rule(lhs).rhs([rhs]);
                // Left recursive
                // seq ::= seq sep item
                if let Proper(sep) = separator {
                    let orig = self.history.as_ref().unwrap().bottom(rhs, Some(sep), &[lhs, sep, rhs]);
                    self.rule(lhs).rhs_with_history([lhs, sep, rhs], orig);
                } else {
                    let orig = self.history.as_ref().unwrap().bottom(rhs, None, &[lhs, rhs]);
                    self.rule(lhs).rhs_with_history([lhs, rhs], orig);
                }
            }
            (_, 1, Some(1)) => {
                self.rule(lhs).rhs([rhs]);
            }
            (_, 1, Some(2)) => {
                let sym1 = self.recurse(sequence.clone().inclusive(1, Some(1)));
                let sym2 = self.recurse(sequence.clone().inclusive(2, Some(2)));
                // seq ::= sym1 | sym2
                self.rule(lhs).rhs([sym1])
                              .rhs([sym2]);
            }
            (separator, 1, Some(end)) => {
                let pow2 = end.next_power_of_two() / 2;
                let (seq1, seq2) = (sequence.clone().inclusive(start, Some(pow2)),
                                    sequence.clone().inclusive(start, Some(end - pow2)));
                let rhs = &[self.recurse(seq1.separator(separator.prefix_separator())),
                            self.recurse(seq2.separator(separator))];
                // seq ::= sym1 sym2
                self.rule(lhs).rhs(rhs);
            }
            // Bug in rustc. Must use comparison.
            (Proper(sep), start, end) if start == 2 && end == Some(2) => {
                let orig = self.history.as_ref().unwrap().bottom(rhs, Some(sep), &[rhs, sep, rhs]);
                self.rule(lhs).rhs_with_history([rhs, sep, rhs], orig);
            }
            (separator, 2 ... 0xFFFF_FFFF, end) => {
                // to do infinity
                let (seq1, seq2) = if Some(start) == end {
                    // A "block"
                    let pow2 = start.next_power_of_two() / 2;
                    (sequence.clone().inclusive(pow2, Some(pow2)),
                     sequence.clone().inclusive(start - pow2, Some(start - pow2)))
                } else {
                    // A "span"
                    (sequence.clone().inclusive(start, Some(start)),
                     sequence.clone().inclusive(1, end.map(|n| n - start - 1)))
                };
                let rhs = &[self.recurse(seq1.separator(separator.prefix_separator())),
                            self.recurse(seq2.separator(separator))];
                // seq ::= sym1 sym2
                self.rule(lhs).rhs(rhs);
            }
            _ => panic!()
        }
    }
}

/// Clone history.
pub struct CloneHistory<'a, H: 'a, S> {
    history: &'a H,
    marker: PhantomData<S>,
}

impl<'a, H, S> CloneHistory<'a, H, S> {
    /// Creates history factory.
    pub fn new(history: &'a H) -> Self {
        CloneHistory {
            history: history,
            marker: PhantomData
        }
    }
}

impl<'a, H, S> HistoryFn<H, S> for CloneHistory<'a, H, S> where
            H: Clone,
            S: GrammarSymbol {
    fn call_mut(&mut self, _lhs: S, _rhs: &[S]) -> H {
        self.history.clone()
    }
}
