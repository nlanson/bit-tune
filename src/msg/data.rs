// data.rs
//
// Message data module
//
//
// General design for Network Messages in this program:
//    Data structures:
//      - Message struct representing an entire network message including the payload.
//      - Message header struct representing a network message header. This will contain
//        the TCP packet magic, command bytes, payload length indicator and checksum.
//      - Magic enum containing magic values for the different compatible networks
//        (mainnet and testnet)
//      - Command enum listing the different commands that the program can read.
//      - Varint struct to create and parse variable length integers.
//
//    Message creation:
//      - Messages can be created via the `Message::new()` API which is written to
//        to take in a `Payload` enum value.
//      - Once the `Message` struct is created, the message will be encoded through a
//        encoding trait which will be implemented for the various data structures.
//
//    Message propagation:
//      - The encoded message can be written into an open TCP stream with a peer.
//
//    Message Serialization/Deserialization:
//      - The trait `Encode` and `Decode` will be implemented for necessary data
//        structures for modular encoding/decoding of network messages.
//      - Message payloads will be decoded as byte dumps instead of being interpreted as 
//        structured data.

use crate::{
    msg::header::{
        MessageHeader,
        Magic,
        Command,
        Checksum
    },
    msg::network::{
        VersionMessage,
        NetAddrTS
    },
    msg::inventory::{
        InvVect
    },
    encode::Encode
};


#[derive(Debug, Clone)]
/// Network message structure
pub struct Message {
    pub header: MessageHeader,
    pub payload: MessagePayload
}

impl Message {
    pub fn new(payload: MessagePayload, magic: Magic, command: Command) -> Message {
        Self {
            header: MessageHeader::new(magic, command, payload.len(), payload.checksum()),
            payload
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Enum that contians the data structures for network messages
pub enum MessagePayload {
    Version(VersionMessage),
    PingPong(u64),
    AddrList(Vec<NetAddrTS>),
    InvVect(Vec<InvVect>),
    
    // Generic payloads for:
    EmptyPayload,   // Payloads with no data
    Dump(Vec<u8>)   // Unknown structure payloads
}

impl MessagePayload {
    /// Get the length of the encoded payload by encoding the
    /// message and returning the length of the encoded message.
    pub fn len(&self) -> usize {
        match self {
            // Payloads with a known fixed size:
            Self::EmptyPayload => 0,
            Self::PingPong(_) => 8,

            // Payloads with a variable size:
            _ => self.net_encode(Vec::new())
        }
    }
}


#[derive(Debug, Clone)]
/// Abstract structure to represent message payloads that hold nothing
pub struct EmptyPayload;

impl Default for EmptyPayload {
    fn default() -> Self {
        Self
    }
}

/// Macro to create a Payload enum from a struct.
macro_rules! payload_from_struct {
    ($struct: ty, $var: ident) => {
        impl From<$struct> for MessagePayload {
            fn from(payload: $struct) -> Self {
                Self::$var(payload)
            }
        }
    };
}

payload_from_struct!(VersionMessage, Version);