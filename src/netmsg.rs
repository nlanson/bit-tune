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

use crate::netmsgheader::{
    MessageHeader,
    Magic,
    Command
};
use sha2::{Sha256, Digest};


/// Network message structure
#[derive(Debug, Clone)]
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

/// Enum that contians the data structures for network messages
#[derive(Debug, Clone)]
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
pub struct VersionMessage {
    version: u32,
    services: NodeServiceFlags,
    timestamp: u64,
    // Todo: Network address structs for the following fields
    // https://en.bitcoin.it/wiki/Protocol_documentation#Network_address
    //
    // addr_recv: NetworkAddress,
    // addr_sent: NetworkAddress
    nonce: u64,
    agent: String,
    start_height: u32,
    relay: bool
}

impl VersionMessage {
    pub fn new(
        version: u32,
        services: NodeServiceFlags,
        timestamp: u64,
        // addr_recv: NetworkAddress,
        // addr_sent: NetworkAddress,
        nonce: u64,
        agent: String,
        start_height: u32,
        relay: bool
    ) -> VersionMessage {
        Self {
            version,
            services,
            timestamp,
            // addr_recv,
            // addr_sent,
            nonce,
            agent,
            start_height,
            relay
        }
    }
}


/// Node services flag to indicate what services are available on a node.
/// Todo: Multiflag support
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NodeServiceFlags {
    None,
    Network,
    GetUTXO,
    Bloom,
    Witness,
    CompactFilters,
    NetworkLimited
}

impl NodeServiceFlags {
    pub fn value(&self) -> u64 {
        match self {
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