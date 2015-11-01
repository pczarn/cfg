use std::mem;
use std::ops::Deref;
use std::slice;

use binarized::BinarizedCfg;
use history::{Binarize, AssignPrecedence, RewriteSequence, NullHistory, Action};
use precedence::PrecedencedRuleBuilder;
use rule::{GrammarRule, Rule};
use rule::builder::RuleBuilder;
use rule::container::RuleContainer;
use sequence::Sequence;
use sequence::builder::SequenceRuleBuilder;
use sequence::rewrite::SequencesToProductions;
use symbol::{ConsecutiveSymbols, SymbolSource, GrammarSymbol, TerminalSymbolSet};

/// Trait for context-free grammars.
pub trait ContextFree: RuleContainer + Sized {
    /// The type of the source of nonterminal symbols for all manipulations on the grammar.
    type Source: SymbolSource<Symbol=Self::Symbol>;

    /// Returns an immutable reference to the grammar's symbol source.
    fn sym_source(&self) -> &Self::Source;

    /// Starts building a new rule.
    fn rule(&mut self, lhs: Self::Symbol) -> RuleBuilder<&mut Self> {
        RuleBuilder::new(self).rule(lhs)
    }

    /// Starts building a new precedenced rule.
    fn precedenced_rule(&mut self, lhs: Self::Symbol) -> PrecedencedRuleBuilder<&mut Self> where
                Self::History: AssignPrecedence + Default {
        PrecedencedRuleBuilder::new(self, lhs)
    }

    /// Returns a binarized weak equivalent of this grammar.
    fn binarize<'a>(&'a self) -> BinarizedCfg<Self::History, Self::Source> where
                &'a Self: ContextFreeRef<'a, Target=Self>,
                Self::History: Binarize + Clone + 'static,
                Self::Source: Clone {
        BinarizedCfg::from_context_free(self)
    }
}

// Traits for working around the lack of higher-order type constructors, more commonly known as HKT
// or HKP.

/// This trait is currently needed to make the associated `Rules` iterator generic over a lifetime
/// parameter.
pub trait ContextFreeRef<'a>: Deref where Self::Target: ContextFree {
    /// Immutable reference to a rule.
    type RuleRef: GrammarRule<Symbol=<<Self as Deref>::Target as SymbolSource>::Symbol,
                              History=<<Self as Deref>::Target as RuleContainer>::History>
                  + Copy + 'a;
    /// Iterator over immutable references to the grammar's rules.
    type Rules: Iterator<Item=Self::RuleRef>;
    /// Returns an iterator over immutable references to the grammar's rules.
    fn rules(self) -> Self::Rules;
}

/// Allows access to a ContextFreeRef through mutable references.
pub trait ContextFreeMut<'a>: Deref where
            Self::Target: ContextFree + 'a,
            &'a Self::Target: ContextFreeRef<'a, Target=Self::Target> {
}

/// Basic representation of context-free grammars.
#[derive(Clone)]
pub struct Cfg<H = NullHistory, Hs = H, Ss = ConsecutiveSymbols> where Ss: SymbolSource {
    /// The symbol source.
    sym_source: Ss,
    /// The array of rules.
    rules: Vec<Rule<H, Ss::Symbol>>,
    /// The array of sequence rules.
    sequence_rules: Vec<Sequence<Hs, Ss::Symbol>>,
}

impl<H, Hs> Cfg<H, Hs> {
    /// Creates an empty context-free grammar.
    pub fn new() -> Cfg<H, Hs> {
        Cfg::with_sym_source(ConsecutiveSymbols::new())
    }
}

impl<H, Hs, Ss> Cfg<H, Hs, Ss> where Ss: SymbolSource {
    /// Creates an empty context-free grammar with the given symbol source.
    pub fn with_sym_source(sym_source: Ss) -> Cfg<H, Hs, Ss> {
        Cfg {
            sym_source: sym_source,
            rules: vec![],
            sequence_rules: vec![],
        }
    }
}

impl<H: Action, Hs, Ss> Cfg<H, Hs, Ss>
        where Hs: RewriteSequence<Rewritten=H>,
              H: Clone,
              Ss: SymbolSource {
    /// Starts building a sequence rule.
    pub fn sequence(&mut self, lhs: Ss::Symbol)
                -> SequenceRuleBuilder<Hs, &mut Vec<Sequence<Hs, Ss::Symbol>>, Ss::Symbol> {
        SequenceRuleBuilder::new(&mut self.sequence_rules).sequence(lhs)
    }

    /// Returns sequence rules.
    pub fn sequence_rules(&self) -> &[Sequence<Hs, Ss::Symbol>] {
        &self.sequence_rules
    }

    /// Forces a rewrite of sequence rules into grammar rules.
    pub fn rewrite_sequences(&mut self) {
        let sequence_rules = mem::replace(&mut self.sequence_rules, vec![]);
        SequencesToProductions::rewrite_sequences(&sequence_rules[..], self);
    }
}

impl<H: Action, Hs, Ss> ContextFree for Cfg<H, Hs, Ss> where
            Ss: SymbolSource,
            Hs: Clone + RewriteSequence<Rewritten=H> {
    type Source = Ss;

    fn sym_source(&self) -> &Ss {
        &self.sym_source
    }

    fn binarize<'a>(&'a self) -> BinarizedCfg<Self::History, Self::Source> where
                &'a Self: ContextFreeRef<'a, Target=Self>,
                H: Binarize + Clone + 'static,
                Ss: Clone {
        let mut grammar = BinarizedCfg::from_context_free(self);
        SequencesToProductions::rewrite_sequences(&self.sequence_rules[..], &mut grammar);
        grammar
    }
}

impl<'a, H: Action, Hs, Ss> ContextFreeRef<'a> for &'a Cfg<H, Hs, Ss> where
            H: 'a,
            Hs: Clone + RewriteSequence<Rewritten=H>,
            Ss: SymbolSource + 'a,
            Ss::Symbol: 'a {
    type RuleRef = <Self::Rules as Iterator>::Item;
    type Rules = slice::Iter<'a, Rule<H, Ss::Symbol>>;

    fn rules(self) -> Self::Rules {
        self.rules.iter()
    }
}

impl<'a, H: Action, Hs, Ss> ContextFreeMut<'a> for &'a mut Cfg<H, Hs, Ss> where
            H: 'a,
            Hs: Clone + RewriteSequence<Rewritten=H> + 'a,
            Ss: SymbolSource + 'a,
            Ss::Symbol: 'a {
}

impl<H, Hs, Ss> RuleContainer for Cfg<H, Hs, Ss> where
            Ss: SymbolSource,
            Ss::Symbol: GrammarSymbol {
    type History = H;

    fn retain<F>(&mut self, mut f: F) where
                F: FnMut(Self::Symbol, &[Self::Symbol], &Self::History) -> bool {
        self.rules.retain(|rule| f(rule.lhs(), rule.rhs(), rule.history()));
    }

    fn add_rule(&mut self, lhs: Self::Symbol,
                           rhs: &[Self::Symbol],
                           history: H) {
        self.sym_source.mark_as_nonterminal(lhs);
        self.rules.push(Rule::new(lhs, rhs.to_vec(), history));
    }
}

impl<H, Hs, Ss> SymbolSource for Cfg<H, Hs, Ss> where Ss: SymbolSource {
    type Symbol = Ss::Symbol;

    fn next_sym(&mut self, terminal: bool) -> Self::Symbol {
        self.sym_source.next_sym(terminal)
    }

    fn mark_as_nonterminal(&mut self, sym: Self::Symbol) {
        self.sym_source.mark_as_nonterminal(sym)
    }

    fn num_syms(&self) -> usize {
        self.sym_source.num_syms()
    }
}

impl<H, Hs, Ss> TerminalSymbolSet for Cfg<H, Hs, Ss> where Ss: TerminalSymbolSet {
    fn is_terminal(&self, sym: Self::Symbol) -> bool {
        self.sym_source.is_terminal(sym)
    }
}
