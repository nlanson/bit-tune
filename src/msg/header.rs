// headers.rs
//
// Module for network message headers
//
//

use crate::encode::{Encode, Error};
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
    Unknown(u32)
}

impl Magic {
    /// Return the magic bytes for the specified network.
    pub fn bytes(&self) -> u32 {
        match self {
            Magic::Main => 0xD9B4BEF9,
            Magic::Test => 0xDAB5BFFA,
            Magic::Unknown(v)=> *v
        }
    }
}

impl From<[u8; 4]> for Magic {
    fn from(bytes: [u8; 4]) -> Self {
        if bytes == Magic::Main.bytes().to_be_bytes() { return Magic::Main }
        else if bytes == Magic::Test.bytes().to_be_bytes() { return Magic::Test }
        else { return Magic::Unknown(
            bytes[0] as u32 >> 24 |
            bytes[1] as u32 >> 16 |
            bytes[2] as u32 >> 8 |
            bytes[3] as u32
        ) }
    }
}



#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
/// Network command enum
//  Adding a new command requires:
//     - A new entry in the payload enum (Or reuse the same payload for a different command)
//     - Associated match statements modified to support the new command/payload.
pub enum Command {
    Version,
    Verack,
    SendHeaders,
    WTxIdRelay,
    Ping,
    Pong,
    Addr,
    GetAddr,
    Inv,
    GetData,
    NotFound,
    Tx,
    //More to come...

    // Command enum option for unknonwn/invalid command strings
    Unknown(String)
    
}

impl Command {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Version => "version",
            Self::Verack =>  "verack",
            Self::SendHeaders => "sendheaders",
            Self::WTxIdRelay => "wtxidrelay",
            Self::Ping => "ping",
            Self::Pong => "pong",
            Self::Addr => "addr",
            Self::GetAddr => "getaddr",
            Self::Inv => "inv",
            Self::GetData => "getdata",
            Self::NotFound => "notfound",
            Self::Tx => "tx",
            Self::Unknown(s) => &s
        }
    }

    pub fn from_str(cmd: String) -> Result<Self, Error> {
        match &cmd[..] {
            "version" => Ok(Self::Version),
            "verack" => Ok(Self::Verack),
            "sendheaders" => Ok(Self::SendHeaders),
            "wtxidrelay" => Ok(Self::WTxIdRelay),
            "ping" => Ok(Self::Ping),
            "pong" => Ok(Self::Pong),
            "addr" => Ok(Self::Addr),
            "getaddr" => Ok(Self::GetAddr),
            "inv" => Ok(Self::Inv),
            "getdata" => Ok(Self::GetData),
            "notfound" => Ok(Self::NotFound),
            "tx" => Ok(Self::Tx),
            _ => Err(Error::UnknownCommand(cmd))
        }
    }
}


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

impl<T: Encode> Checksum for T {
    fn checksum(&self) -> [u8; 4] {
        let mut ret: [u8; 4] = [0; 4];
        let mut payload = Vec::new();
        self.net_encode(&mut payload);

        ret.copy_from_slice(&sha256d(payload)[..4]);
        ret
    }
}