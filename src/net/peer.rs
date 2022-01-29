// net.rs
//
// Module involving networking code.
//

use crate::{
    msg::network::NetAddress
};
use crate::net::Error;
use rayon::prelude::*;
use std::net::{
    Ipv4Addr,
    TcpStream
};

#[derive(Copy, Clone, Debug)]
pub struct Peer {
    pub addr: Ipv4Addr,
    pub port: Port
}

impl Peer {
    /// Get a list of working peers
    pub fn get(min: usize, peerlist: &[[u8; 6]]) -> Result<Vec<Self>, Error> {
        // Get a list of potential peers from the seeds module
        let mut ut_peers: Vec<UntestedPeer> = peerlist
            .iter()
            .map(|x| UntestedPeer::from(*x))
            .collect::<Vec<UntestedPeer>>();

        // While the minium number of peers is not met and there
        // are peers to test remaining, paralell test if a peer
        // is active or not.
        let mut peers: Vec<Peer> = vec![];
        while peers.len() < min || ut_peers.len() < min {
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

        // If the minimum amount of connections could not be made, return an error.
        if peers.len() < min {
            return Err(Error::FailedToConnect(String::from("Failed to establish minimum peer connections")))
        }

        Ok(peers)
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

impl From<NetAddress> for Peer {
    fn from(netaddr: NetAddress) -> Peer {
        Peer {
            addr: match netaddr.address.ip() {
                std::net::IpAddr::V4(x) => x,
                std::net::IpAddr::V6(_) => panic!("Peer struct does not support IPv6")
            },
            port: Port::from(netaddr.address.port())
        }
    }
}

impl std::string::ToString for Peer {
    fn to_string(&self) -> String {
        format!("{}:{}", self.addr.to_string(), self.port.to_u16())
    }
}

/// Type alias for distinguishing between tested and untested peers.
pub type UntestedPeer = Peer; 

impl From<[u8; 6]> for UntestedPeer {
    fn from(seed: [u8; 6]) -> Self {
        Self {
            addr: Ipv4Addr::from([seed[0], seed[1], seed[2], seed[3]]),
            port: Port::from([seed[4], seed[5]])
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// TCP/IP Port stored as big endian bytes
//  Hence the use of [u8; 2] instead of u16.
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