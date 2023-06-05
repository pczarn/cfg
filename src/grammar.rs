use std::mem;
use std::ops::Deref;
use std::slice;

use binarized::BinarizedCfg;
use history::{AssignPrecedence, Binarize, NullHistory, RewriteSequence};
use precedence::PrecedencedRuleBuilder;
use rule::builder::RuleBuilder;
use rule::container::{EmptyRuleContainer, RuleContainer};
use rule::{GrammarRule, Rule};
use sequence::builder::SequenceRuleBuilder;
use sequence::rewrite::SequencesToProductions;
use sequence::Sequence;
use symbol::source::SymbolContainer;
use symbol::{Symbol, SymbolSource};

/// Trait for context-free grammars.
pub trait ContextFree: RuleContainer + Sized {
    /// Starts building a new rule.
    fn rule(&mut self, lhs: Symbol) -> RuleBuilder<&mut Self> {
        RuleBuilder::new(self).rule(lhs)
    }

    /// Starts building a new precedenced rule.
    fn precedenced_rule(&mut self, lhs: Symbol) -> PrecedencedRuleBuilder<&mut Self>
    where
        Self::History: AssignPrecedence + Default,
    {
        PrecedencedRuleBuilder::new(self, lhs)
    }
}

// Traits for working around the lack of higher-order type constructors, more commonly known as HKT
// or HKP.

/// This trait is currently needed to make the associated `Rules` iterator generic over a lifetime
/// parameter.
pub trait ContextFreeRef<'a>: Deref + Sized
where
    Self::Target: ContextFree,
{
    /// Immutable reference to a rule.
    type RuleRef: GrammarRule<History = <<Self as Deref>::Target as RuleContainer>::History>
        + Copy
        + 'a;
    /// Iterator over immutable references to the grammar's rules.
    type Rules: Iterator<Item = Self::RuleRef>;
    /// Returns an iterator over immutable references to the grammar's rules.
    fn rules(self) -> Self::Rules;

    /// Reverses the grammar.
    fn reverse(self) -> Self::Target
    where
        <Self::Target as RuleContainer>::History: Clone,
        Self::Target: EmptyRuleContainer,
    {
        let mut new_grammar = (*self).empty();
        for _ in 0..self.sym_source().num_syms() {
            let _: Symbol = new_grammar.sym();
        }
        for rule in self.rules() {
            let mut rhs = rule.rhs().iter().cloned().collect::<Vec<_>>();
            rhs.reverse();
            new_grammar.add_rule(rule.lhs(), &rhs[..], (*rule.history()).clone());
        }
        new_grammar
    }
}

/// Allows access to a ContextFreeRef through mutable references.
pub trait ContextFreeMut<'a>: Deref
where
    Self::Target: ContextFree + 'a,
    &'a Self::Target: ContextFreeRef<'a, Target = Self::Target>,
{
}

/// Basic representation of context-free grammars.
#[derive(Clone)]
pub struct Cfg<H = NullHistory, Hs = H> {
    /// The symbol source.
    sym_source: SymbolSource,
    /// The array of rules.
    rules: Vec<Rule<H>>,
    /// The array of sequence rules.
    sequence_rules: Vec<Sequence<Hs>>,
}

impl<H, Hs> Default for Cfg<H, Hs> {
    fn default() -> Self {
        Self::with_sym_source(SymbolSource::new())
    }
}

impl<H, Hs> Cfg<H, Hs> {
    /// Creates an empty context-free grammar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty context-free grammar with the given symbol source.
    pub fn with_sym_source(sym_source: SymbolSource) -> Self {
        Cfg {
            sym_source: sym_source,
            rules: vec![],
            sequence_rules: vec![],
        }
    }
}

impl<H, Hs> Cfg<H, Hs>
where
    Hs: RewriteSequence<Rewritten = H>,
    H: Clone,
    Hs: Clone,
{
    /// Returns generated symbols.
    pub fn sym<T>(&mut self) -> T
    where
        T: SymbolContainer,
    {
        self.sym_source_mut().sym()
    }

    /// Generates a new unique symbol.
    pub fn next_sym(&mut self) -> Symbol {
        self.sym_source_mut().next_sym()
    }

    /// Returns the number of symbols in use.
    pub fn num_syms(&self) -> usize {
        self.sym_source().num_syms()
    }

    /// Starts building a sequence rule.
    pub fn sequence(&mut self, lhs: Symbol) -> SequenceRuleBuilder<Hs, &mut Vec<Sequence<Hs>>> {
        SequenceRuleBuilder::new(&mut self.sequence_rules).sequence(lhs)
    }

    /// Returns sequence rules.
    pub fn sequence_rules(&self) -> &[Sequence<Hs>] {
        &self.sequence_rules
    }

    /// Forces a rewrite of sequence rules into grammar rules.
    pub fn rewrite_sequences(&mut self) {
        let sequence_rules = mem::replace(&mut self.sequence_rules, vec![]);
        SequencesToProductions::rewrite_sequences(&sequence_rules[..], self);
    }

    /// Returns a binarized grammar which is weakly equivalent to this grammar.
    pub fn binarize<'a>(&'a self) -> BinarizedCfg<H>
    where
        &'a Self: ContextFreeRef<'a, Target = Self>,
        H: Binarize + Clone + 'static,
    {
        let mut grammar = BinarizedCfg::from_context_free(self);
        SequencesToProductions::rewrite_sequences(&self.sequence_rules[..], &mut grammar);
        grammar
    }
}

impl<H, Hs> ContextFree for Cfg<H, Hs> where Hs: Clone + RewriteSequence<Rewritten = H> {}

impl<'a, H, Hs> ContextFreeRef<'a> for &'a Cfg<H, Hs>
where
    H: 'a,
    Hs: Clone + RewriteSequence<Rewritten = H>,
{
    type RuleRef = <Self::Rules as Iterator>::Item;
    type Rules = slice::Iter<'a, Rule<H>>;

    fn rules(self) -> Self::Rules {
        self.rules.iter()
    }
}

impl<'a, H, Hs> ContextFreeMut<'a> for &'a mut Cfg<H, Hs>
where
    H: 'a,
    Hs: Clone + RewriteSequence<Rewritten = H> + 'a,
{
}

impl<H, Hs> RuleContainer for Cfg<H, Hs>
where
    Hs: Clone + RewriteSequence<Rewritten = H>,
{
    type History = H;

    fn sym_source(&self) -> &SymbolSource {
        &self.sym_source
    }

    fn sym_source_mut(&mut self) -> &mut SymbolSource {
        &mut self.sym_source
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(Symbol, &[Symbol], &H) -> bool,
    {
        self.rules
            .retain(|rule| f(rule.lhs(), rule.rhs(), rule.history()));
    }

    fn add_rule(&mut self, lhs: Symbol, rhs: &[Symbol], history: H) {
        self.rules.push(Rule::new(lhs, rhs.to_vec(), history));
    }
}

impl<H, Hs> EmptyRuleContainer for Cfg<H, Hs>
where
    Hs: Clone + RewriteSequence<Rewritten = H>,
{
    fn empty(&self) -> Self {
        Cfg::default()
    }
}
