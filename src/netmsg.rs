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

// Network message structure
#[derive(Debug, Clone)]
pub struct Message {
    header: MessageHeader,
    payload: Vec<u8>
}

// Message header structure
#[derive(Debug, Clone)]
pub struct MessageHeader {
    magic: Magic,
    command: Command,
    length: u32,
    checksum: u32
}

// Network magic enum
#[derive(Debug, Clone)]
pub enum Magic {
    // Todo: Add mainnet and testnet magic bytes
}

// Network command enum
#[derive(Debug, Clone)]
pub enum Command {
    // Todo: Add basic commands
}

// Variable length integer structure
#[derive(Debug, Clone)]
pub struct VariableInteger(u64);


pub trait Serialise {
    /// Encodes self into a format acceptable by the Bitcoin P2P network.
    fn consensus_serialise<W>(&self, &mut buf: W) -> usize
    where W: std::io::Write;
}