#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Id(u32);

const NULL_ID: Id = Id(u32::MAX);

impl Id {
    #[inline]
    pub fn usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn to_option(self) -> Option<u32> {
        if self == NULL_ID {
            None
        } else {
            Some(self.0)
        }
    }
}

impl From<Option<u32>> for Id {
    fn from(value: Option<u32>) -> Self {
        value.map(Id).unwrap_or(NULL_ID)
    }
}
