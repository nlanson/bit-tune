// btcnetmsg-lib
//
// A small library for encoding/decoding bitcoin P2P messages
//
//
//  Todos:
//  - Implement other common network messages (block, headers, etc...)
//
//  - Use rust-bitcoin crate for blockdata
//
//  - Message structures refactor to enforce payload restrictions based on command type.



// Modules
pub mod msg;
pub mod encode;
pub mod blockdata;
pub mod address;

// Re-exports
pub use bitcoin as bitcoin;
pub use msg::{
    data::{
        Message,
        MessagePayload
    },
    header::{
        MessageHeader,
        Magic,
        Command
    },

    network::{
        VersionMessage,
        ServicesList,
        Service
    },
    inventory::Inventory
};
pub use encode::{
    Encode,
    Decode,
    Error
};
pub use address::Address;