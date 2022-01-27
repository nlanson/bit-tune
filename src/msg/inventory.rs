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
pub struct InvVect {
    pub dtype: InvObjectType,
    pub hash: Hash
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvObjectType {
    Error,
    Tx,
    Block,
    FilteredBlock,
    CompactBlock,
    WitnessTx,
    WitnessBlock,
    FilteredWitnessBlock
}

impl InvObjectType {
    pub fn value(&self) -> u32 {
        match self {
            Self::Error => 0,
            Self::Tx => 1,
            Self::Block => 2,
            Self::FilteredBlock => 3,
            Self::CompactBlock => 4,
            Self::WitnessTx => 0x40000001,
            Self::WitnessBlock => 0x40000002,
            Self::FilteredWitnessBlock => 0x40000003
        }
    }

    pub fn from_u32(value: u32) -> Result<Self, Error> {
        Ok(match value {
            0 => Self::Error,
            1 => Self::Tx,
            2 => Self::Block,
            3 => Self::FilteredBlock,
            4 => Self::CompactBlock,
            0x40000001 => Self::WitnessTx,
            0x40000002 => Self::WitnessBlock,
            0x40000003 => Self::FilteredWitnessBlock,
            _ => return Err(Error::InvalidData)
        })
    }
}