// headers.rs
//
// Module for network message headers
//
//

use crate::msg::{
    data::{
        MessagePayload
    }
};
use crate::encode::Error;
use sha2::{
    Sha256, Digest
};

/// Message header structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    pub magic: Magic,
    pub command: Command,
    pub length: u32,
    pub checksum: [u8; 4]
}

impl MessageHeader {
    pub fn new(magic: Magic, command: Command, pylen: usize, checksum: [u8; 4]) -> MessageHeader {
        Self {
            magic,
            command,
            length: pylen as u32,
            checksum
        }
    }
}

/// Network magic enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Magic {
    Main,
    Test,
    Unknown
}

impl Magic {
    /// Return the magic bytes for the specified network.
    pub fn bytes(&self) -> u32 {
        match self {
            Magic::Main => 0xD9B4BEF9,
            Magic::Test => 0xDAB5BFFA,
            _ =>           0x00000000
        }
    }
}

impl From<[u8; 4]> for Magic {
    fn from(bytes: [u8; 4]) -> Self {
        if bytes == Magic::Main.bytes().to_be_bytes() { return Magic::Main }
        else if bytes == Magic::Test.bytes().to_be_bytes() { return Magic::Test }
        else { return Magic::Unknown }
    }
}


/// Network command enum
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Command {
    Version,
    Verack
    //More to come...
}

impl Command {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Version => "version",
            Self::Verack =>  "verack"
        }
    }

    pub fn from_str(cmd: String) -> Result<Self, Error> {
        match &cmd[..] {
            "version" => Ok(Self::Version),
            "verack" => Ok(Self::Verack),
            _ => Err(Error::InvalidData)
        }
    }
}

impl From<&MessagePayload> for Command {
    fn from(payload: &MessagePayload) -> Self {
        match payload {
            MessagePayload::Version(_) => Command::Version,
            MessagePayload::Verack(_) => Command::Verack
        }
    }
}

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


pub trait Checksum {
    fn checksum(&self) -> [u8; 4];
}

/// Sha256d convenience function
pub fn sha256d<T: AsRef<[u8]>>(data: T) -> [u8; 32] {
    let mut ret: [u8; 32] = [0; 32];
    let mut o = Sha256::new();
    let mut i = Sha256::new();

    i.update(data);
    o.update(i.finalize());
    
    ret.copy_from_slice(&o.finalize()[..]);
    ret
}