// stream.rs
//
// Module for TCP related code.
//

use crate::net::{
    peer::{
        Peer
    },
    Error
};
use std::net::{
    TcpStream
};

impl From<Peer> for Result<TcpStream, Error> {
    /// Create a tcp stream from a peer
    fn from(peer: Peer) -> Result<TcpStream, Error> {
        match TcpStream::connect(peer.to_string()) {
            Ok(x) => Ok(x),
            Err(_) => Err(Error::FailedToConnect(peer.to_string()))
        }
    }
}