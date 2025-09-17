//! Remaps symbols and removes unused symbols.

use std::cmp::Ordering;

use cfg_symbol::SymbolSource;
use log::trace;

use cfg_grammar::{Cfg, CfgRule};
use cfg_symbol::Symbol;
use cfg_symbol::intern::Mapping;

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
                to_internal: SymbolSource::generate_fresh()
                    .take(num_syms)
                    .map(Some)
                    .collect(),
                to_external: SymbolSource::generate_fresh().take(num_syms).collect(),
            },
        }
    }

    /// Removes unused symbols.
    pub fn remove_unused_symbols(&mut self) {
        let unused_symbols = self.grammar.unused_symbols();
        self.reorder_symbols(|sym0, sym1| unused_symbols[sym0].cmp(&unused_symbols[sym1]));
        let num_syms = self.grammar.num_syms();
        self.grammar
            .sym_source_mut()
            .truncate(num_syms - unused_symbols.iter().count());
    }

    /// Remaps symbols to satisfy given ordering constraints. The argument
    /// must be a function that gives total order.
    pub fn reorder_symbols<F>(&mut self, f: F)
    where
        F: Fn(Symbol, Symbol) -> Ordering,
    {
        // Create a new map from N to N symbols.
        let mut new_mapping = Mapping::new(self.grammar.num_syms());
        new_mapping.to_external = SymbolSource::generate_fresh()
            .take(self.grammar.num_syms())
            .collect();
        // Sort its external symbols (corresponding to internal symbols of self.maps)
        // according to the given order.
        new_mapping.to_external.sort_by(|&a, &b| f(a, b));
        // Update its internal symbol map based on external symbol map.
        for (before, after) in new_mapping
            .to_external
            .iter()
            .cloned()
            .zip(SymbolSource::generate_fresh())
        {
            new_mapping.to_internal[before.usize()] = Some(after);
        }
        self.mapping.translate(&new_mapping);
        self.remap_symbols(|sym| new_mapping.to_internal[sym.usize()].unwrap());
    }

    // Translates symbols in rules to new symbol IDs.
    pub fn remap_symbols<F>(&mut self, mut map: F)
    where
        F: FnMut(Symbol) -> Symbol,
    {
        let mut added_rules = vec![];
        self.grammar.retain(|rule| {
            trace!(
                "REMAP {:?} -> {:?} ::= {:?} -> {:?}",
                rule.lhs,
                map(rule.lhs),
                rule.rhs,
                rule.rhs.iter().map(|&sym| map(sym)).collect::<Vec<_>>()
            );
            if map(rule.lhs) == rule.lhs && rule.rhs.iter().all(|&sym| map(sym) == sym) {
                true
            } else {
                added_rules.push(CfgRule {
                    lhs: map(rule.lhs),
                    rhs: rule.rhs.iter().cloned().map(&mut map).collect(),
                    history: rule.history,
                });
                false
            }
        });
        for rule in added_rules {
            self.grammar.add_rule(rule);
        }

        // let mut translate = |root: Symbol| {
        //     if let Some(internal_start) = maps.to_internal[root.usize()] {
        //         internal_start
        //     } else {
        //         // FIXME: weird
        //         println!("removing {:?}", root);
        //         // The trivial grammar is a unique edge case -- the start symbol was removed.
        //         let internal_start = Symbol::from(maps.to_external.len());
        //         maps.to_internal[root.usize()] = Some(internal_start);
        //         maps.to_external.push(root);
        //         internal_start
        //     }
        // };
        let roots: Vec<_> = self.grammar.roots().iter().copied().map(&mut map).collect();
        self.grammar.set_roots(&roots[..]);
        let wrapped_roots: Vec<_> = self
            .grammar
            .wrapped_roots()
            .iter()
            .copied()
            .map(|mut wrapped_root| {
                trace!(
                    "REMAP WRAPPED ROOT {:?}",
                    (
                        wrapped_root.inner_root,
                        wrapped_root.root,
                        wrapped_root.start_of_input,
                        wrapped_root.end_of_input
                    )
                );
                wrapped_root.inner_root = map(wrapped_root.inner_root);
                wrapped_root.root = map(wrapped_root.root);
                wrapped_root.start_of_input = map(wrapped_root.start_of_input);
                wrapped_root.end_of_input = map(wrapped_root.end_of_input);
                wrapped_root
            })
            .collect();
        self.grammar.set_wrapped_roots(&wrapped_roots[..]);
        self.grammar.sym_source_mut().remap_symbols(map);
    }

    /// Get the mapping.
    pub fn get_mapping(self) -> Mapping {
        self.mapping
    }
}

pub trait CfgRemapSymbolsExt {
    fn remap(&mut self) -> Remap<'_>;
}

impl CfgRemapSymbolsExt for Cfg {
    fn remap(&mut self) -> Remap<'_> {
        Remap::new(self)
    }
}
