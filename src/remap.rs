//! Remaps symbols and removes unused symbols.

use std::cmp::Ordering;
use std::iter;
use std::mem;
use std::ops;

use crate::prelude::*;
use crate::rule::{GrammarRule, Rule};

/// Remaps symbols and removes unused symbols.
pub struct Remap<'a, G: 'a>
where
    G: RuleContainer,
{
    grammar: &'a mut G,
    mapping: Mapping,
}

/// Populates maps with new symbols.
struct Intern {
    source: SymbolSource,
    mapping: Mapping,
}

/// Contains maps for translation between internal and external symbols.
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Mapping {
    /// An array of internal symbols, indexed by external symbol ID.
    pub to_internal: Vec<Option<Symbol>>,
    /// An array of external symbols, indexed by internal symbol ID.
    pub to_external: Vec<Symbol>,
}

impl<'a, G> Remap<'a, G>
where
    G: RuleContainer,
{
    /// Creates `Remap` to record information about remapped symbols.
    pub fn new(grammar: &'a mut G) -> Self {
        let num_syms = grammar.num_syms();
        Remap {
            grammar: grammar,
            mapping: Mapping {
                to_internal: symbol_iter(num_syms).map(Some).collect(),
                to_external: symbol_iter(num_syms).collect(),
            },
        }
    }

    /// Removes unused symbols.
    pub fn remove_unused_symbols(&mut self) {
        let mut intern = Intern::new(self.grammar.num_syms());
        self.remap_symbols(|sym| intern.intern(sym));
        let _ = mem::replace(self.grammar.sym_source_mut(), intern.source);
        self.mapping.translate(&intern.mapping);
    }

    /// Remaps symbols to satisfy given ordering constraints. The argument
    /// must be a function that gives total order.
    pub fn reorder_symbols<F>(&mut self, f: F)
    where
        F: Fn(Symbol, Symbol) -> Ordering,
    {
        // Create a new map from N to N symbols.
        let mut new_mapping = Mapping::new(self.grammar.num_syms());
        new_mapping.to_external = symbol_iter(self.grammar.num_syms()).collect();
        // Sort its external symbols (corresponding to internal symbols of self.maps)
        // according to the given order.
        new_mapping.to_external.sort_by(|&a, &b| f(a, b));
        // Update its internal symbol map based on external symbol map.
        for (after_id, before) in new_mapping.to_external.iter().cloned().enumerate() {
            let after = Symbol::from(after_id);
            new_mapping.to_internal[before.usize()] = Some(after);
        }
        self.mapping.translate(&new_mapping);
        self.remap_symbols(|sym| new_mapping.to_internal[sym.usize()].unwrap());
    }

    // Translates symbols in rules to new symbol IDs.
    fn remap_symbols<F>(&mut self, mut map: F)
    where
        F: FnMut(Symbol) -> Symbol,
    {
        let mut added_rules = vec![];
        self.grammar.retain(|lhs, rhs, history| {
            if map(lhs) == lhs && rhs.iter().all(|&sym| map(sym) == sym) {
                true
            } else {
                added_rules.push(Rule::new(
                    map(lhs),
                    rhs.iter().cloned().map(&mut map).collect(),
                    history,
                ));
                false
            }
        });
        for rule in added_rules {
            let rule_lhs = rule.lhs();
            self.grammar
                .add_rule(rule_lhs, &rule.rhs[..], rule.history_id());
        }
    }

    /// Get the mapping.
    pub fn get_mapping(self) -> Mapping {
        self.mapping
    }
}

impl Intern {
    fn new(num_external: usize) -> Self {
        Intern {
            source: SymbolSource::new(),
            mapping: Mapping::new(num_external),
        }
    }

    fn intern(&mut self, symbol: Symbol) -> Symbol {
        if let Some(internal) = self.mapping.to_internal[symbol.usize()] {
            internal
        } else {
            let new_sym = self.source.sym();
            self.mapping.to_internal[symbol.usize()] = Some(new_sym);
            assert_eq!(self.mapping.to_external.len(), new_sym.usize());
            self.mapping.to_external.push(symbol);
            new_sym
        }
    }
}

impl Mapping {
    /// Creates a new instance of `Mapping`.
    pub fn new(num_external: usize) -> Self {
        Mapping {
            to_internal: vec![None; num_external],
            to_external: vec![],
        }
    }

    /// Translates symbols in this map using another symbol map.
    /// This map becomes a combination of both mappings.
    pub fn translate(&mut self, other: &Self) {
        // For mapping to internal.
        for internal in &mut self.to_internal[..] {
            *internal = if let Some(sym) = *internal {
                other.to_internal[sym.usize()]
            } else {
                None
            };
        }
        // For mapping to external.
        let remapped = other
            .to_external
            .iter()
            .map(|middle| self.to_external[middle.usize()])
            .collect();
        self.to_external = remapped;
    }
}

// Iterates over symbols with consecutive IDs.
fn symbol_iter(num: usize) -> iter::Map<ops::Range<usize>, fn(usize) -> Symbol> {
    (0..num).map(Symbol::from)
}
