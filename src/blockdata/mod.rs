// blockdata/mod.rs
//
// Blockdata module
//
//

//Hashes are encoded in little endian
pub type Hash = [u8; 32];
pub const GENESIS_HASH: Hash = [0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f];


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