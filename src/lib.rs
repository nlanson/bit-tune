// btcnetmsg-lib
//
// A small library for encoding/decoding bitcoin P2P messages
//
//
//  Todos:
//  - Implement other common network messages (getblocks, getheaders, getdata, tx, block, headers)
//
//  - Use rust-bitcoin crate for blockdata
//
//  - Move bit-tune application code into a new crate


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