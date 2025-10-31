//! Utility for string interning.

use elsa::FrozenIndexSet;

/// Collects strings.
pub struct StringInterner {
    set: FrozenIndexSet<String>,
}

impl StringInterner {
    /// Creates a new `StringInterner`.
    pub fn new() -> Self {
        StringInterner {
            set: FrozenIndexSet::new(),
        }
    }

    /// Retrieves an interned value, or inserts a new entry
    /// if it does not exist.
    pub fn get_or_intern<T>(&self, value: T) -> usize
    where
        T: AsRef<str>,
    {
        // TODO use Entry in case the standard Entry API gets improved
        // (here to avoid premature allocation or double lookup)
        self.set.insert_full(value.as_ref().to_string()).0
    }

    // fn get<T>(&self, value: T) -> Option<usize>
    // where
    //     T: AsRef<str>,
    // {
    //     self.set.get_full(value.as_ref()).map(|(i, _r)| i)
    // }

    pub fn resolve(&self, index: usize) -> Option<&str> {
        self.set.get_index(index)
    }
}
