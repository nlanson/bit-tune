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

// Network command enum
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
            Self::Verack => "verack"
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