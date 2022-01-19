// net/mod.rs
//
// Bundling module for networking
//

pub mod peer;
pub mod stream;

#[derive(Debug)]
pub enum Error {
    FailedToConnect(String),
}