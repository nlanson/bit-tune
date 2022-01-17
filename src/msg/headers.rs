// headers.rs
//
// Module for network message headers
//
//


/// Message header structure
#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub magic: Magic,
    pub command: Command,
    pub length: VariableInteger,
    pub checksum: [u8; 4]
}

impl MessageHeader {
    pub fn new(magic: Magic, command: Command, pylen: usize, checksum: [u8; 4]) -> MessageHeader {
        Self {
            magic,
            command,
            length: VariableInteger::from(pylen),
            checksum
        }
    }
}

/// Network magic enum
#[derive(Debug, Clone)]
pub enum Magic {
    Main,
    Test
}

impl Magic {
    /// Return the magic bytes for the specified network.
    pub fn bytes(&self) -> u32 {
        match self {
            Magic::Main => 0xD9B4BEF9,
            Magic::Test => 0xDAB5BFFA
        }
    }
}


/// Network command enum
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Command {
    Version,
    Verack,
    //More to come...
}

impl Command {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Version => "version",
            Self::Verack =>  "verack"
        }
    }
}


// Variable length integer structure
#[derive(Debug, Clone)]
pub struct VariableInteger(pub u64);

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