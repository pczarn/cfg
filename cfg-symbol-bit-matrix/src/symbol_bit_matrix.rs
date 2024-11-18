use std::cmp;
use std::ops::{self, Deref, DerefMut};

use bit_matrix::BitMatrix;

use cfg_grammar::Cfg;
use cfg_symbol::intern::Mapping;
use cfg_symbol::{Symbol, Symbolic};

use crate::Remap;

pub struct SymbolBitMatrix {
    bit_matrix: BitMatrix,
}

pub struct DirectDerivationMatrix(SymbolBitMatrix);
pub struct ReachabilityMatrix(SymbolBitMatrix);
pub struct UnitDerivationMatrix(SymbolBitMatrix);

impl SymbolBitMatrix {
    fn new(num_syms: usize) -> Self {
        SymbolBitMatrix {
            bit_matrix: BitMatrix::new(num_syms, num_syms),
        }
    }

    fn set(&mut self, row: Symbol, col: Symbol, included: bool) {
        self.bit_matrix.set(row.usize(), col.usize(), included);
    }

    pub fn iter_row_syms(&self, row: Symbol) -> impl Iterator<Item = Symbol> + '_ {
        self.bit_matrix
            .iter_row(row.usize())
            .enumerate()
            .filter_map(|(i, present)| if present { Some(i.into()) } else { None })
    }
}

impl Deref for SymbolBitMatrix {
    type Target = BitMatrix;
    fn deref(&self) -> &Self::Target {
        &self.bit_matrix
    }
}

impl DerefMut for SymbolBitMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bit_matrix
    }
}

impl Deref for DirectDerivationMatrix {
    type Target = SymbolBitMatrix;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DirectDerivationMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for ReachabilityMatrix {
    type Target = SymbolBitMatrix;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReachabilityMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for UnitDerivationMatrix {
    type Target = SymbolBitMatrix;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnitDerivationMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<DirectDerivationMatrix> for SymbolBitMatrix {
    fn from(value: DirectDerivationMatrix) -> Self {
        value.0
    }
}

impl From<ReachabilityMatrix> for SymbolBitMatrix {
    fn from(value: ReachabilityMatrix) -> Self {
        value.0
    }
}

impl From<UnitDerivationMatrix> for SymbolBitMatrix {
    fn from(value: UnitDerivationMatrix) -> Self {
        value.0
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

impl ops::Index<(Symbol, Symbol)> for SymbolBitMatrix {
    type Output = bool;
    fn index(&self, index: (Symbol, Symbol)) -> &Self::Output {
        if self.bit_matrix[(index.0.usize(), index.1.usize())] {
            &TRUE
        } else {
            &FALSE
        }
    }
}

impl TryFrom<BitMatrix> for SymbolBitMatrix {
    type Error = ();

    fn try_from(bit_matrix: BitMatrix) -> Result<Self, Self::Error> {
        let (rows, cols) = bit_matrix.size();
        if rows == cols {
            Ok(SymbolBitMatrix { bit_matrix })
        } else {
            Err(())
        }
    }
}

impl From<SymbolBitMatrix> for BitMatrix {
    fn from(value: SymbolBitMatrix) -> Self {
        value.bit_matrix
    }
}

impl ReachabilityMatrix {
    pub fn reflexive(mut self) -> Self {
        self.reflexive_closure();
        self
    }
}

impl DirectDerivationMatrix {
    /// Returns the derivation matrix.
    pub fn reachability(mut self) -> ReachabilityMatrix {
        self.transitive_closure();
        ReachabilityMatrix(self.into())
    }
}

pub trait CfgSymbolBitMatrixExt {
    fn empty_matrix(&self) -> SymbolBitMatrix;
    fn direct_derivation_matrix(&self) -> DirectDerivationMatrix;
    fn reachability_matrix(&self) -> ReachabilityMatrix;
    fn unit_derivation_matrix(&self) -> UnitDerivationMatrix;
    fn reorder_symbols(&mut self, order: SymbolBitMatrix) -> Mapping;
    fn remove_unused_symbols(&mut self);
}

impl CfgSymbolBitMatrixExt for Cfg {
    fn empty_matrix(&self) -> SymbolBitMatrix {
        SymbolBitMatrix::new(self.num_syms())
    }

    fn direct_derivation_matrix(&self) -> DirectDerivationMatrix {
        let mut derivation = self.empty_matrix();

        for rule in self.rules() {
            for &sym in rule.rhs.iter() {
                derivation.set(rule.lhs, sym, true);
            }
        }

        DirectDerivationMatrix(derivation)
    }

    fn reachability_matrix(&self) -> ReachabilityMatrix {
        self.direct_derivation_matrix().reachability()
    }

    fn unit_derivation_matrix(&self) -> UnitDerivationMatrix {
        let mut unit_derivation = self.empty_matrix();

        for rule in self.rules() {
            // A rule of form `A ::= A` is not a cycle. We can represent unit rules in the form of
            // a directed graph. The rule `A ::= A` is then presented as a self-loop. Self-loops
            // aren't cycles.
            if rule.rhs.len() == 1 && rule.lhs != rule.rhs[0] {
                unit_derivation.set(rule.lhs, rule.rhs[0], true);
            }
        }

        unit_derivation.transitive_closure();
        UnitDerivationMatrix(unit_derivation)
    }

    fn reorder_symbols(&mut self, mut order: SymbolBitMatrix) -> Mapping {
        // the order above is not transitive.
        // We modify it so that if `A < B` and `B < C` then `A < C`
        order.transitive_closure();
        let mut maps = {
            let mut remap = Remap::new(&mut *self);
            remap.remove_unused_symbols();
            remap.reorder_symbols(|left, right| {
                if order[(left, right)] {
                    cmp::Ordering::Less
                } else if order[(right, left)] {
                    cmp::Ordering::Greater
                } else {
                    cmp::Ordering::Equal
                }
            });
            remap.get_mapping()
        };
        let roots: Vec<_> = self
            .roots()
            .iter()
            .copied()
            .map(|root| {
                if let Some(internal_start) = maps.to_internal[root.usize()] {
                    internal_start
                } else {
                    // The trivial grammar is a unique edge case -- the start symbol was removed.
                    let internal_start = Symbol::from(maps.to_external.len());
                    maps.to_internal[root.usize()] = Some(internal_start);
                    maps.to_external.push(root);
                    internal_start
                }
            })
            .collect();
        self.set_roots(&roots[..]);
        maps
    }

    fn remove_unused_symbols(&mut self) {
        Remap::new(self).remove_unused_symbols();
    }
}
