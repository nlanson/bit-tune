// net.rs
//
// Module involving networking code.
//
//
use rayon::prelude::*;
use crate::seeds::ipv4bitseeds;
use std::net::{
    Ipv4Addr,
    TcpStream
};

#[derive(Copy, Clone, Debug)]
pub struct Peer {
    pub addr: Ipv4Addr,
    pub port: Port
}
pub type UntestedPeer = Peer; //Type alias for distinguishing between tested and untested peers.

impl From<[u8; 6]> for UntestedPeer {
    fn from(seed: [u8; 6]) -> Self {
        Self {
            addr: Ipv4Addr::from([seed[0], seed[1], seed[2], seed[3]]),
            port: Port::from([seed[4], seed[5]])
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// TCP/IP Port stored as big endian bytes
pub struct Port(pub [u8; 2]);

impl From<u16> for Port {
    fn from(port: u16) -> Port {
        Port(port.to_be_bytes())
    }
}

impl From<[u8; 2]> for Port {
    fn from(port: [u8; 2]) -> Port {
        Port(port)
    }
}

impl Port {
    pub fn to_u16(&self) -> u16 {
        ((self.0[0] as u16) << 8) | (self.0[1] as u16)
    }
}

impl Peer {
    /// Get a list of working peers
    pub fn get(min: usize) -> Vec<Self> {
        // Get a list of potential peers from the seeds module
        let mut ut_peers: Vec<UntestedPeer> = ipv4bitseeds
            .iter()
            .map(|x| UntestedPeer::from(*x))
            .collect::<Vec<UntestedPeer>>();

        // While the peer list is not the minimum length,
        // take a chunk of untested peers, test that they work,
        // and then push the working peers into the peers list.
        // Then remove the peers that have been tested from the
        // untested peers list. 
        let mut peers: Vec<Peer> = vec![];
        while peers.len() < min {
            peers.extend_from_slice(
                &ut_peers
                    .par_iter()                                            // Paralell test
                    .take(num_cpus::get())                            // 1 peer per CPU core (Rayon spawns 1 thread per core.)
                    .map(|x| {
                        if x.test_conn() { return Some(*x) }                         // If peer works, save it as Some
                        None                                                         // else dont save it
                    })
                    .filter(|x| x.is_some()) // Remove None peers from the list
                    .map(|x| x.unwrap())
                    .collect::<Vec<Peer>>()                                          // Collect working peers to push
            );
            ut_peers.drain(0..min);                                            // Remove tested peers
        }

        peers
    }
    
    /// Test if a peer is accepting TCP connections
    fn test_conn(&self) -> bool {
        let peer: String = format!("{}:{}", self.addr.to_string(), self.port.to_u16());

        if let Ok(_) = TcpStream::connect(&peer) {
            println!("Connection established to {}", peer);
            return true
        }

        println!("Failed to connect to {}", peer);
        false
    }
}