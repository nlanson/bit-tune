// netmsg.rs
//
// Module for Bitcoin network messages.
//
//
// General design for Network Messages in this program:
//    Data structures:
//      - Message struct representing an entire network message including the payload.
//      - Message header struct representing a network message header. This will contain
//        the TCP packet magic, command bytes, payload length indicator and checksum.
//      - Magic enum containing magic values for the different compatible networks
//        (mainnet and testnet)
//      - Command enum listing the different commands that the program can read. The
//        program will not be able to constuct certain network messages such as the 
//        `inv` message as it does not store any blockdata.
//      - Varint struct to create and parse variable length integers.
//
//    Message creation:
//      - Messages can be created via the `Message::create()` API which will be written to
//        to take in a `Command` enum value. The program will fail if the selected command is
//        not a supported command.
//      - Once the `Message` struct is created, the message will be serialized through a
//        serialisation trait which will be implemented for the various data structures.
//
//    Message propagation:
//      - Serialized messages will be passed to the net module for TCP stream writing.
//
//    Message Serialization/Deserialization:
//      - The trait `Serialise` and `Deserialise` will be implemented for necessary data
//        structures for modular encoding/decoding of network messages.
//      - Message payloads will not be deserialized for unsupported network messages

use crate::{
    netmsgheader::{
        MessageHeader,
        Magic,
        Command
    },
    net::Port
};
use std::{net::Ipv4Addr, collections::HashSet};
use sha2::{Sha256, Digest};


#[derive(Debug, Clone)]
/// Network message structure
pub struct Message {
    pub header: MessageHeader,
    pub payload: MessagePayload
}

impl Message {
    pub fn new(payload: MessagePayload, magic: Magic, command: Command) -> Message {
        Self {
            header: MessageHeader::new(magic, command, payload.len(), payload.hash()),
            payload
        }
    }
}


#[derive(Debug, Clone)]
/// Enum that contians the data structures for network messages
pub enum MessagePayload {
    Version(VersionMessage),
    Verack
}

impl MessagePayload {
    /// Get the length of the encoded payload
    pub fn len(&self) -> usize {
        todo!();
    }

    /// Hash the payload to create the checksum
    pub fn hash(&self) -> [u8; 4] {
        todo!();
    }
}


#[derive(Debug, Clone)]
/// The message payload for version commands.
pub struct VersionMessage {
    pub version: u32,
    pub services: ServicesList,
    pub timestamp: std::time::SystemTime,
    pub addr_recv: NetAddr,
    pub addr_sent: NetAddr,
    pub nonce: u64,
    pub agent: String,
    pub start_height: u32,
    pub relay: bool
}

impl VersionMessage {
    pub fn new(
        version: u32,
        services: ServicesList,
        timestamp: std::time::SystemTime,
        addr_recv: NetAddr,
        addr_sent: NetAddr,
        nonce: u64,
        agent: String,
        start_height: u32,
        relay: bool
    ) -> VersionMessage {
        Self {
            version,
            services,
            timestamp,
            addr_recv,
            addr_sent,
            nonce,
            agent,
            start_height,
            relay
        }
    }
}



#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
#[allow(dead_code)]
/// Node services flag to indicate what services are available on a node.
/// Todo: Multiflag support
pub enum Services {
    None,
    Network,
    GetUTXO,
    Bloom,
    Witness,
    CompactFilters,
    NetworkLimited
}

impl Services {
    pub fn value(&self) -> u64 {
        match self {
            // Each service is a bit flag
            Self::None => 0,                // No services available
            Self::Network => 1,             // Full chain history available
            Self::GetUTXO => 2,             // Can be queried for UTXOs
            Self::Bloom => 4,               // Capable of handling bloom filtered connections
            Self::Witness => 8,             // Witness data available
            Self::CompactFilters => 64,     // Can serve basic block filte requests
            Self::NetworkLimited => 1024    // Can serve blocks from the last 2 days
        }
    }
}

#[derive(Clone, Debug)]
/// A list of service flags in a hash set.
/// DOES NOT ENFORCE CONFLICTING FLAGS
pub struct ServicesList(std::collections::HashSet<Services>);

impl ServicesList {
    pub fn new() -> Self {
        ServicesList(HashSet::new())
    }

    pub fn add_flag(&mut self, flag: Services) {
        self.0.insert(flag);
    }

    pub fn get_flags(&self) -> Vec<Services> {
        self.0.iter().map(|flag| *flag).collect()
    }
}

impl Default for ServicesList {
    fn default() -> Self {
        let mut flags = Self::new();
        flags.add_flag(Services::None);
        flags
    }
}



#[derive(Clone, Debug)]
/// When a network address is needed somewhere, this structure is used.
pub struct NetAddr {
    pub services: ServicesList,
    pub ip: Ipv4Addr,
    pub port: Port,
}

impl NetAddr {
    pub fn new(
        services: ServicesList,
        ip: Ipv4Addr,
        port: Port
    ) -> Self {
        Self {
            services,
            ip: ip,
            port: port
        }
    }
}

impl Default for NetAddr {
    fn default() -> NetAddr {
        Self {
            services: ServicesList::new(),
            ip: Ipv4Addr::new(0, 0, 0, 0),
            port: Port::from(0)
        }
    }
}

impl From<(Ipv4Addr, Port)> for NetAddr {
    fn from(net_info: (Ipv4Addr, Port)) -> NetAddr {
        Self {
            services: ServicesList::new(),
            ip: net_info.0,
            port: net_info.1
        }
    }
}