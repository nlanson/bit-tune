// inventory.rs
//
// Module for `inv` and related network messages
//
//

pub use crate::bitcoin::{
    hash_types::{
        Txid,
        BlockHash
    },
    hashes::Hash
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inventory {
    // If an inv value has this flag, ignore it
    Error,
    // Hash of a regular TXID
    Tx(Txid), 
    // Hash of a block
    Block(BlockHash),
    // Only used in getdata message. Indicates reply should be merkleblock rather than block
    FilteredBlock(BlockHash),
    // Only used in getdata message. Indicates reply should be cmpctblock rather than block
    CompactBlock(BlockHash),
    // Hash of a TX with witness data
    WitnessTx(Txid),
    // Hash of a block with witness data
    WitnessBlock(BlockHash),
    // Only used in getdata message. Indicates a reply should be merkleblock rather than block
    FilteredWitnessBlock(BlockHash),
    // Unknown hash type
    Unknown {
        inv_type: u32,
        hash: [u8; 32]
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
            Self::Unknown{inv_type, ..} => *inv_type
        }
    }

    /// Creates self from a u32 identified and hash
    pub fn from_id_and_hash(identifier: u32, hash: [u8; 32]) -> Self {
        match identifier {
            0 => Self::Error,
            1 => Self::Tx(Txid::from_inner(hash)),
            2 => Self::Block(BlockHash::from_inner(hash)),
            3 => Self::FilteredBlock(BlockHash::from_inner(hash)),
            4 => Self::CompactBlock(BlockHash::from_inner(hash)),
            0x40000001 => Self::WitnessTx(Txid::from_inner(hash)),
            0x40000002 => Self::WitnessBlock(BlockHash::from_inner(hash)),
            0x40000003 => Self::FilteredWitnessBlock(BlockHash::from_inner(hash)),
            x => Self::Unknown { inv_type: x, hash }
        }
        
    }

    /// Return the inner hash stored in Self.
    /// Returns [0; 32] for error variant.
    pub fn inner(&self) -> [u8; 32] {
        match self {
            Self::Error => [0; 32],
            Self::Tx(x) => x.into_inner(),
            Self::Block(x) => x.into_inner(),
            Self::FilteredBlock(x) => x.into_inner(),
            Self::CompactBlock(x) => x.into_inner(),
            Self::WitnessTx(x) => x.into_inner(),
            Self::WitnessBlock(x) => x.into_inner(),
            Self::FilteredWitnessBlock(x) => x.into_inner(),
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

        write!(f, "INV: [{}] {}", obj_type, self.inner().iter().rev().map(|x| format!("{:02x}", x)).collect::<String>())
    }
}