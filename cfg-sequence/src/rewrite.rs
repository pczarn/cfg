//! Rewrites sequence rules into production rules.

// #[cfg(feature = "nightly")]
// use collections::range::RangeArgument;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::builder::SequenceRuleBuilder;
use crate::destination::SequenceDestination;
use crate::Separator::{self, Liberal, Proper, Trailing};
use crate::Sequence;
use crate::Symbol;
use cfg_grammar::rule_builder::RuleBuilder;
use cfg_grammar::Cfg;
use cfg_history::{earley::History, HistoryNodeRewriteSequence, HistoryNodeSequenceRhs, RootHistoryNode};

/// Rewrites sequence rules into production rules.
pub struct SequencesToProductions<'a> {
    destination: &'a mut Cfg,
    stack: Vec<Sequence>,
    map: HashMap<PartialSequence, Symbol>,
    top: Option<History>,
    lhs: Option<Symbol>,
}

// A key into a private map.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PartialSequence {
    rhs: Symbol,
    start: u32,
    end: Option<u32>,
    separator: Separator,
}

impl<'a> SequenceDestination for SequencesToProductions<'a> {
    fn add_sequence(&mut self, seq: Sequence) {
        self.rewrite(seq);
    }
}

impl From<Sequence> for PartialSequence {
    fn from(value: Sequence) -> Self {
        PartialSequence {
            rhs: value.rhs,
            start: value.start,
            end: value.end,
            separator: value.separator,
        }
    }
}

impl<'a> SequencesToProductions<'a> {
    /// Initializes a rewrite.
    pub fn new(destination: &'a mut Cfg) -> Self {
        SequencesToProductions {
            destination,
            stack: vec![],
            map: HashMap::new(),
            top: None,
            lhs: None,
        }
    }

    /// Rewrites a sequence rule.
    pub fn rewrite(&mut self, top: Sequence) {
        self.stack.clear();
        self.map.clear();
        let prev = top.history.unwrap_or_else(|| {
            RootHistoryNode::NoOp.into()
        });
        let history_top = HistoryNodeRewriteSequence {
                top: true,
                rhs: top.rhs,
                sep: top.separator.into(),
                prev,
            }
            .into();
        self.top = Some(history_top);
        self.reduce(top);
        let prev = top.history.unwrap_or_else(|| {
            RootHistoryNode::NoOp.into()
        });
        let history_bottom = HistoryNodeRewriteSequence {
                top: false,
                rhs: top.rhs,
                sep: top.separator.into(),
                prev,
            }
            .into();
        *self.top.as_mut().unwrap() = history_bottom;
        while let Some(seq) = self.stack.pop() {
            assert!(seq.start <= seq.end.unwrap_or(!0));
            self.reduce(seq);
        }
    }

    fn recurse(&mut self, seq: &Sequence) -> Symbol {
        let sym_source = &mut self.destination;
        // As a placeholder
        let partial: PartialSequence = (*seq).into();

        match self.map.entry(partial) {
            Entry::Vacant(vacant) => {
                let lhs = sym_source.next_sym(None);
                vacant.insert(lhs);
                self.stack.push(Sequence { lhs, ..*seq });
                lhs
            }
            Entry::Occupied(lhs) => *lhs.get(),
        }
    }

    fn rhs<A: AsRef<[Symbol]>>(&mut self, rhs: A) {
        assert!(rhs.as_ref().len() <= 3);
        let history = HistoryNodeSequenceRhs {
                prev: self.top.unwrap(),
                rhs: [rhs.as_ref().get(0).copied(), rhs.as_ref().get(1).copied(), rhs.as_ref().get(2).copied()]
            }.into();
        RuleBuilder::new(self.destination)
            .rule(self.lhs.unwrap())
            .history(history)
            .rhs(rhs);
    }

    fn reduce(&mut self, sequence: Sequence) {
        let Sequence {
            lhs,
            rhs,
            start,
            end,
            separator,
            ..
        } = sequence;
        self.lhs = Some(lhs);
        // TODO optimize reductions
        match (separator, start, end) {
            (Liberal(sep), _, _) => {
                let sym1 = self.recurse(&sequence.separator(Proper(sep)));
                let sym2 = self.recurse(&sequence.separator(Trailing(sep)));
                // seq ::= sym1 | sym2
                self.rhs([sym1]);
                self.rhs([sym2]);
            }
            (_, 0, Some(0)) => {
                // seq ::= epsilon | sym
                self.rhs([]);
            }
            (_, 0, end) => {
                // seq ::= epsilon | sym
                self.rhs([]);
                let sym = self.recurse(&sequence.inclusive(1, end));
                self.rhs([sym]);
            }
            (Trailing(sep), _, _) => {
                let sym = self.recurse(&sequence.separator(Proper(sep)));
                // seq ::= sym sep
                self.rhs([sym, sep]);
            }
            (_, 1, None) => {
                // ???
                // seq ::= item
                self.rhs([rhs]);
                // Left recursive
                // seq ::= seq sep item
                if let Proper(sep) = separator {
                    self.rhs([lhs, sep, rhs]);
                } else {
                    self.rhs([lhs, rhs]);
                }
            }
            (_, 1, Some(1)) => {
                self.rhs([rhs]);
            }
            (_, 1, Some(2)) => {
                let sym1 = self.recurse(&sequence.range(1..=1));
                let sym2 = self.recurse(&sequence.range(2..=2));
                // seq ::= sym1 | sym2
                self.rhs([sym1]);
                self.rhs([sym2]);
            }
            (_, 1, Some(end)) => {
                // end >= 3
                let pow2 = end.next_power_of_two() / 2;
                let (seq1, block, seq2) = (
                    sequence.range(1..=pow2),
                    sequence.range(pow2..=pow2),
                    sequence.range(1..=end - pow2),
                );
                let rhs1 = self.recurse(&seq1);
                let block = self.recurse(&block.separator(separator.prefix_separator()));
                let rhs2 = self.recurse(&seq2);
                // seq ::= sym1 sym2
                self.rhs([rhs1]);
                self.rhs([block, rhs2]);
            }
            (Proper(sep), 2, Some(2)) => {
                self.rhs([rhs, sep, rhs]);
            }
            (_, 2, Some(2)) => {
                self.rhs([rhs, rhs]);
            }
            (_, 2.., end) => {
                // to do infinity
                let (seq1, seq2) = if Some(start) == end {
                    // A "block"
                    let pow2 = start.next_power_of_two() / 2;
                    (
                        sequence.range(pow2..=pow2),
                        sequence.range(start - pow2..=start - pow2),
                    )
                } else {
                    // A "span"
                    (
                        sequence.range(start - 1..=start - 1),
                        sequence.inclusive(1, end.map(|end| end - start + 1)),
                    )
                };
                let (rhs1, rhs2) = (
                    self.recurse(&seq1.separator(separator.prefix_separator())),
                    self.recurse(&seq2.separator(separator)),
                );
                // seq ::= sym1 sym2
                self.rhs([rhs1, rhs2]);
            }
        }
    }
}
