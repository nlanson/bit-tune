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
    msg::headers::{
        MessageHeader,
        Magic,
        Command
    },
    msg::network::{
        VersionMessage,
        VerackMessage
    }, encode::Encode
};
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
    Verack(VerackMessage)
}

impl MessagePayload {
    /// Get the length of the encoded payload by encoding the
    /// message and returning the length of the encoded message.
    pub fn len(&self) -> usize {
        match self {
            Self::Version(v) => v.net_encode(Vec::new()),
            Self::Verack(_) => 0,
        }
    }

    /// Hash the payload to create the checksum
    pub fn hash(&self) -> [u8; 4] {
        todo!();
    }
}