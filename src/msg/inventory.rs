// inventory.rs
//
// Module for `inv` and related network messages
//
//

use crate::{
    blockdata::Hash,
    encode::Error
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inventory {
    Error,
    Tx(Hash),
    Block(Hash),
    FilteredBlock(Hash),
    CompactBlock(Hash),
    WitnessTx(Hash),
    WitnessBlock(Hash),
    FilteredWitnessBlock(Hash),
    Unknown {
        inv_type: u32,
        hash: Hash
    }
}

impl Inventory {
    /// Return the u32 identifier of self
    pub fn identifier(&self) -> u32 {
        match self {
            Self::Error => 0,
            Self::Tx(_) => 1,
            Self::Block(_) => 2,
            Self::FilteredBlock(_) => 3,
            Self::CompactBlock(_) => 4,
            Self::WitnessTx(_) => 0x40000001,
            Self::WitnessBlock(_) => 0x40000002,
            Self::FilteredWitnessBlock(_) => 0x40000003,
            Self::Unknown{inv_type, hash: _} => *inv_type
        }
    }

    /// Creates self from a u32 identified and hash
    pub fn from_id_and_hash(identifier: u32, hash: Hash) -> Self {
        match identifier {
            0 => Self::Error,
            1 => Self::Tx(hash),
            2 => Self::Block(hash),
            3 => Self::FilteredBlock(hash),
            4 => Self::CompactBlock(hash),
            0x40000001 => Self::WitnessTx(hash),
            0x40000002 => Self::WitnessBlock(hash),
            0x40000003 => Self::FilteredWitnessBlock(hash),
            x => Self::Unknown { inv_type: x, hash }
        }
        
    }

    /// Return the inner hash stored in Self.
    /// Returns [0; 32] for error variant.
    pub fn inner(&self) -> Hash {
        match self {
            Self::Error => [0; 32],
            Self::Tx(x) => *x,
            Self::Block(x) => *x,
            Self::FilteredBlock(x) => *x,
            Self::CompactBlock(x) => *x,
            Self::WitnessTx(x) => *x,
            Self::WitnessBlock(x) => *x,
            Self::FilteredWitnessBlock(x) => *x,
            Self::Unknown{inv_type: _, hash} => *hash
        }
    }
}

impl std::fmt::Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let obj_type = match self {
            Self::Error => "Error",
            Self::Tx(_) => "Transaction",
            Self::Block(_) => "Block",
            Self::FilteredBlock(_) => "Filtered Block",
            Self::CompactBlock(_) => "Compact Block",
            Self::WitnessTx(_) => "Witness Transaction",
            Self::WitnessBlock(_) => "Witness Block",
            Self::FilteredWitnessBlock(_) => "Filtered Witness Block",
            Self::Unknown{inv_type: _, hash: _} => "Unknown"
        };

        write!(f, "Inv({}: {})", obj_type, self.inner().iter().rev().map(|x| format!("{:02x}", x)).collect::<String>())
    }
}