//! Remaps symbols and removes unused symbols.

use std::cmp::Ordering;
use std::iter;
use std::mem;
use std::ops;

use cfg_symbol::intern::{Intern, Mapping};

use crate::local_prelude::*;

/// Remaps symbols and removes unused symbols.
pub struct Remap<'a> {
    grammar: &'a mut Cfg,
    mapping: Mapping,
}

impl<'a> Remap<'a> {
    /// Creates `Remap` to record information about remapped symbols.
    pub fn new(grammar: &'a mut Cfg) -> Self {
        let num_syms = grammar.num_syms();
        Remap {
            grammar,
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
        self.grammar.retain(|rule| {
            if map(rule.lhs) == rule.lhs && rule.rhs.iter().all(|&sym| map(sym) == sym) {
                true
            } else {
                added_rules.push(CfgRule {
                    lhs: map(rule.lhs),
                    rhs: rule.rhs.iter().cloned().map(&mut map).collect(),
                    history_id: rule.history_id,
                });
                false
            }
        });
        for rule in added_rules {
            self.grammar.add_rule(rule);
        }
    }

    /// Get the mapping.
    pub fn get_mapping(self) -> Mapping {
        self.mapping
    }
}

// Iterates over symbols with consecutive IDs.
fn symbol_iter(num: usize) -> iter::Map<ops::Range<usize>, fn(usize) -> Symbol> {
    (0..num).map(Symbol::from)
}
