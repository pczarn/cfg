use crate::Cfg;

impl Cfg {
    /// Compares the grammars. The symbol numeric values must match.
    pub fn equivalent(&self, other: &Cfg) -> bool {
        let mut eq = true;
        for (i, j) in self.rules().zip(other.rules()) {
            eq = eq && i.lhs == j.lhs && i.rhs.iter().zip(j.rhs.iter()).all(|(a, b)| *a == *b);
        }
        eq
    }
}
