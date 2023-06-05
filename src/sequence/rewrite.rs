//! Rewrites sequence rules into production rules.

// #[cfg(feature = "nightly")]
// use collections::range::RangeArgument;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use history::{HistorySource, NullHistory, RewriteSequence};
use rule::builder::RuleBuilder;
use rule::container::RuleContainer;
use sequence::builder::SequenceRuleBuilder;
use sequence::destination::SequenceDestination;
use sequence::Separator::{Liberal, Proper, Trailing};
use sequence::{Separator, Sequence};
use symbol::Symbol;

/// Rewrites sequence rules into production rules.
pub struct SequencesToProductions<H, D>
where
    H: RewriteSequence,
    D: RuleContainer,
{
    destination: D,
    stack: Vec<Sequence<NullHistory>>,
    map: HashMap<PartialSequence, Symbol>,
    top_history: Option<H>,
    at_top: bool,
    rhs: Option<Symbol>,
    separator: Option<Symbol>,
}

// A key into a private map.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PartialSequence {
    rhs: Symbol,
    start: u32,
    end: Option<u32>,
    separator: Separator,
}

impl<H, D> SequenceDestination<H> for SequencesToProductions<H, D>
where
    D: RuleContainer<History = H::Rewritten>,
    H: Clone + RewriteSequence,
    H::Rewritten: Clone,
{
    fn add_sequence(&mut self, seq: Sequence<H>) {
        self.rewrite(seq);
    }
}

impl<H, D> SequencesToProductions<H, D>
where
    D: RuleContainer<History = H::Rewritten>,
    H: Clone + RewriteSequence,
    H::Rewritten: Clone,
{
    /// Initializes a rewrite.
    pub fn new(destination: D) -> Self {
        SequencesToProductions {
            destination: destination,
            stack: vec![],
            map: HashMap::new(),
            top_history: None,
            at_top: false,
            rhs: None,
            separator: None,
        }
    }

    /// Rewrites sequence rules.
    pub fn rewrite_sequences(sequence_rules: &[Sequence<H>], rules: D) {
        let mut rewrite = SequenceRuleBuilder::new(SequencesToProductions::new(rules));
        for rule in sequence_rules {
            rewrite = rewrite
                .sequence(rule.lhs)
                .separator(rule.separator)
                .inclusive(rule.start, rule.end)
                .rhs_with_history(rule.rhs, &rule.history);
        }
    }

    /// Rewrites a sequence rule.
    pub fn rewrite(&mut self, top: Sequence<H>) {
        self.stack.clear();
        self.map.clear();
        self.top_history = Some(top.history);
        let top = Sequence {
            lhs: top.lhs,
            rhs: top.rhs,
            start: top.start,
            end: top.end,
            separator: top.separator,
            history: NullHistory,
        };
        self.rhs = Some(top.rhs);
        self.separator = top.separator.into();
        self.at_top = true;
        self.reduce(top);
        self.at_top = false;
        while let Some(seq) = self.stack.pop() {
            assert!(seq.start <= seq.end.unwrap_or(!0));
            self.reduce(seq);
        }
    }

    fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut D, DefaultSeqHistory<H>> {
        let default = DefaultSeqHistory {
            top_history: self.top_history.as_ref().unwrap(),
            at_top: self.at_top,
            elem: self.rhs.unwrap(),
            separator: self.separator,
        };
        RuleBuilder::new(&mut self.destination)
            .rule(lhs)
            .default_history(default)
    }

    fn recurse(&mut self, seq: &Sequence<NullHistory>) -> Symbol {
        let sym_source = &mut self.destination;
        // As a placeholder
        let partial = PartialSequence {
            rhs: seq.rhs,
            separator: seq.separator,
            start: seq.start,
            end: seq.end,
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
            Entry::Occupied(lhs) => *lhs.get(),
        }
    }

    fn reduce(&mut self, sequence: Sequence<NullHistory>) {
        let Sequence {
            lhs,
            rhs,
            start,
            end,
            separator,
            ..
        } = sequence;
        // TODO optimize reductions
        match (separator, start, end) {
            (Liberal(sep), _, _) => {
                let sym1 = self.recurse(&sequence.clone().separator(Proper(sep)));
                let sym2 = self.recurse(&sequence.clone().separator(Trailing(sep)));
                // seq ::= sym1 | sym2
                self.rule(lhs).rhs([sym1]).rhs([sym2]);
            }
            (_, _, _) if start == 0 => {
                // seq ::= epsilon | sym
                self.rule(lhs).rhs([]);
                if end != Some(0) {
                    let sym = self.recurse(&sequence.inclusive(1, end));
                    self.rule(lhs).rhs([sym]);
                }
            }
            (Trailing(sep), _, _) => {
                let sym = self.recurse(&sequence.separator(Proper(sep)));
                // seq ::= sym sep
                self.rule(lhs).rhs([sym, sep]);
            }
            (_, _, _) if start == 1 && end == None => {
                if self.at_top {
                    let rec = self.recurse(&sequence);
                    self.rule(lhs).rhs([rec]);
                } else {
                    // seq ::= item
                    self.rule(lhs).rhs([rhs]);
                    // Left recursive
                    // seq ::= seq sep item
                    if let Proper(sep) = separator {
                        self.rule(lhs).rhs([lhs, sep, rhs]);
                    } else {
                        self.rule(lhs).rhs([lhs, rhs]);
                    }
                }
            }
            (_, _, _) if (start, end) == (1, Some(1)) => {
                self.rule(lhs).rhs([rhs]);
            }
            (_, _, _) if (start, end) == (1, Some(2)) => {
                let sym1 = self.recurse(&sequence.clone().inclusive(1, Some(1)));
                let sym2 = self.recurse(&sequence.clone().inclusive(2, Some(2)));
                // seq ::= sym1 | sym2
                self.rule(lhs).rhs([sym1]).rhs([sym2]);
            }
            (_, _, Some(end)) if start == 1 => {
                // end >= 3
                let pow2 = end.next_power_of_two() / 2;
                let (seq1, block, seq2) = (
                    sequence.clone().inclusive(1, Some(pow2)),
                    sequence.clone().inclusive(pow2, Some(pow2)),
                    sequence.clone().inclusive(1, Some(end - pow2)),
                );
                let rhs1 = self.recurse(&seq1);
                let block = self.recurse(&block.separator(separator.prefix_separator()));
                let rhs2 = self.recurse(&seq2);
                // seq ::= sym1 sym2
                self.rule(lhs).rhs([rhs1]).rhs([block, rhs2]);
            }
            // Bug in rustc. Must use comparison.
            (Proper(sep), start, end) if start == 2 && end == Some(2) => {
                self.rule(lhs).rhs([rhs, sep, rhs]);
            }
            (_, _, _) if start == 2 && end == Some(2) => {
                self.rule(lhs).rhs([rhs, rhs]);
            }
            (_, _, end) if start >= 2 => {
                // to do infinity
                let (seq1, seq2) = if Some(start) == end {
                    // A "block"
                    let pow2 = start.next_power_of_two() / 2;
                    (
                        sequence.clone().inclusive(pow2, Some(pow2)),
                        sequence.clone().inclusive(start - pow2, Some(start - pow2)),
                    )
                } else {
                    // A "span"
                    (
                        sequence.clone().inclusive(start - 1, Some(start - 1)),
                        sequence
                            .clone()
                            .inclusive(1, end.map(|end| end - start + 1)),
                    )
                };
                let (rhs1, rhs2) = (
                    self.recurse(&seq1.separator(separator.prefix_separator())),
                    self.recurse(&seq2.separator(separator)),
                );
                // seq ::= sym1 sym2
                self.rule(lhs).rhs([rhs1, rhs2]);
            }
            _ => panic!(),
        }
    }
}

struct DefaultSeqHistory<'a, H: 'a>
where
    H: RewriteSequence,
{
    top_history: &'a H,
    at_top: bool,
    elem: Symbol,
    separator: Option<Symbol>,
}

impl<'a, H> HistorySource<H::Rewritten> for DefaultSeqHistory<'a, H>
where
    H: RewriteSequence,
    H::Rewritten: Clone,
{
    fn build(&mut self, _lhs: Symbol, rhs: &[Symbol]) -> H::Rewritten {
        if self.at_top {
            self.top_history.top(self.elem, self.separator, rhs)
        } else {
            self.top_history.bottom(self.elem, self.separator, rhs)
        }
    }
}
