// msg/mod.rs
//
// Bundling together message related data structures.
//

pub mod data;
pub mod header;
pub mod network;
pub mod inventory;

// Variable length integer structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInteger(pub u64);

impl VariableInteger {
    pub fn inner(&self) -> u64 {
        self.0
    }
}

macro_rules! varint_from {
    ($int: ty) => {
        impl From<$int> for VariableInteger {
            fn from(int: $int) -> VariableInteger {
                VariableInteger(int as u64)
            }
        }
    };
}

varint_from!(u8);
varint_from!(u16);
varint_from!(u32);
varint_from!(u64);
varint_from!(usize);