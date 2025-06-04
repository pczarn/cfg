#[cfg_attr(feature = "miniserde", derive(miniserde::Serialize, miniserde::Deserialize))]
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Id { n: u32 }

const NULL_ID: Id = Id { n: u32::MAX };

impl Id {
    #[inline]
    pub fn usize(self) -> usize {
        self.n as usize
    }

    #[inline]
    pub fn to_option(self) -> Option<u32> {
        if self == NULL_ID {
            None
        } else {
            Some(self.n)
        }
    }
}

impl From<Option<u32>> for Id {
    fn from(value: Option<u32>) -> Self {
        value.map(|n| Id { n }).unwrap_or(NULL_ID)
    }
}
