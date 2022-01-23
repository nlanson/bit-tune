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
    msg::header::{
        MessageHeader,
        Magic,
        Command,
        Checksum
    },
    msg::network::{
        VersionMessage
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
            header: MessageHeader::new(magic, command, payload.len(), payload.hash()),
            payload
        }
    }
}

#[derive(Debug, Clone)]
/// Enum that contians the data structures for network messages
pub enum MessagePayload {
    Version(VersionMessage),
    Verack,
    SendHeaders,
    WTxIdRelay
}

impl MessagePayload {
    /// Get the length of the encoded payload by encoding the
    /// message and returning the length of the encoded message.
    pub fn len(&self) -> usize {
        match self {
            Self::Version(v) => v.net_encode(Vec::new()),
            Self::Verack => 0,
            Self::SendHeaders => 0,
            Self::WTxIdRelay => 0
        }
    }

    /// Hash the payload to create the checksum
    pub fn hash(&self) -> [u8; 4] {
        match self {
            Self::Version(v) => v.checksum(),
            Self::Verack => EmptyPayload.checksum(),
            Self::SendHeaders => EmptyPayload.checksum(),
            Self::WTxIdRelay => EmptyPayload.checksum()
        }
    }
}

#[derive(Debug, Clone)]
/// Abstract structure to represent message payloads that hold nothing
pub struct EmptyPayload;

impl Checksum for EmptyPayload {
    fn checksum(&self) -> [u8; 4] {
        // Sha256d of nothing precomputed:
        [0x5D, 0xF6, 0xE0, 0xE2]
    }
}

impl Default for EmptyPayload {
    fn default() -> Self {
        Self
    }
}

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