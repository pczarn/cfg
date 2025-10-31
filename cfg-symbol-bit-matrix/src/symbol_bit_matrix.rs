use std::ops::{self, Deref, DerefMut};

use bit_matrix::BitMatrix;

use cfg_grammar::Cfg;
use cfg_symbol::{Symbol, SymbolSource};

/// A matrix that represents a relation `R(A, B)` between two symbols.
#[derive(Debug)]
pub struct SymbolBitMatrix {
    bit_matrix: BitMatrix,
}

/// A direct derivation matrix.
pub struct DirectDerivationMatrix(SymbolBitMatrix);
/// A reachability matrix.
pub struct ReachabilityMatrix(SymbolBitMatrix);
/// A unit derivation matrix.
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

    /// Creates an iterator over symbols which appear in the given row.
    pub fn iter_row_syms(&self, row: Symbol) -> impl Iterator<Item = Symbol> + '_ {
        self.bit_matrix
            .iter_row(row.usize())
            .zip(SymbolSource::generate_fresh())
            .filter_map(|(present, sym)| if present { Some(sym) } else { None })
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
    /// A symbol is reachable from itself.
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

/// Extension traits for building matrices that represent relation between symbols,
/// `R(A, B)` where `A`: [`Symbol`], `B`: [`Symbol`].
pub trait CfgSymbolBitMatrixExt {
    /// Creates the empty matrix of size `|S|x|S|` where `S`: set of symbols.
    fn empty_matrix(&self) -> SymbolBitMatrix;
    /// Computes the direct derivation matrix.
    fn direct_derivation_matrix(&self) -> DirectDerivationMatrix;
    /// Computes the reachability matrix.
    fn reachability_matrix(&self) -> ReachabilityMatrix;
    /// Computes the unit derivation matrix.
    /// 
    /// A unit derivation is defined with a grammar rule such as:
    /// ```ignore
    /// A ::= B;
    /// ```
    fn unit_derivation_matrix(&self) -> UnitDerivationMatrix;
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
}
