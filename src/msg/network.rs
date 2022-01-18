// network.rs
//
// Structures related to networking messages (ie NetAddr, VersionMessage, Verack) are
// implemented in this module.
//

use crate::{
    net::peer::Port,
};
use std::net::Ipv4Addr;
use std::collections::HashSet;


#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
#[allow(dead_code)]
/// Node services flag to indicate what services are available on a node.
pub enum Services {
    None,
    Network,
    GetUTXO,
    Bloom,
    Witness,
    CompactFilters,
    NetworkLimited
}

impl Services {
    pub fn value(&self) -> u64 {
        match self {
            // Each service is a bit flag
            Self::None => 0,                // No services available
            Self::Network => 1,             // Full chain history available
            Self::GetUTXO => 2,             // Can be queried for UTXOs
            Self::Bloom => 4,               // Capable of handling bloom filtered connections
            Self::Witness => 8,             // Witness data available
            Self::CompactFilters => 64,     // Can serve basic block filte requests
            Self::NetworkLimited => 1024    // Can serve blocks from the last 2 days
        }
    }
}

#[derive(Clone, Debug)]
/// A list of service flags in a hash set.
/// DOES NOT ENFORCE CONFLICTING FLAGS
pub struct ServicesList(std::collections::HashSet<Services>);

impl ServicesList {
    pub fn new() -> Self {
        ServicesList(HashSet::new())
    }

    pub fn add_flag(&mut self, flag: Services) {
        self.0.insert(flag);
    }

    pub fn get_flags(&self) -> Vec<Services> {
        self.0.iter().map(|flag| *flag).collect()
    }
}

impl Default for ServicesList {
    fn default() -> Self {
        let mut flags = Self::new();
        flags.add_flag(Services::None);
        flags
    }
}



#[derive(Clone, Debug)]
/// When a network address is needed somewhere, this structure is used.
pub struct NetAddr {
    pub services: ServicesList,
    pub ip: Ipv4Addr,
    pub port: Port,
}

impl NetAddr {
    pub fn new(
        services: ServicesList,
        ip: Ipv4Addr,
        port: Port
    ) -> Self {
        Self {
            services,
            ip: ip,
            port: port
        }
    }
}

impl Default for NetAddr {
    fn default() -> NetAddr {
        Self {
            services: ServicesList::new(),
            ip: Ipv4Addr::new(0, 0, 0, 0),
            port: Port::from(0)
        }
    }
}

impl From<(Ipv4Addr, Port)> for NetAddr {
    fn from(net_info: (Ipv4Addr, Port)) -> NetAddr {
        Self {
            services: ServicesList::new(),
            ip: net_info.0,
            port: net_info.1
        }
    }
}


#[derive(Debug, Clone)]
/// The message payload for version commands.
pub struct VersionMessage {
    pub version: u32,
    pub services: ServicesList,
    pub timestamp: std::time::SystemTime,
    pub addr_recv: NetAddr,
    pub addr_sent: NetAddr,
    pub nonce: u64,
    pub agent: String,
    pub start_height: u32,
    pub relay: bool
}

impl VersionMessage {
    pub fn new(
        version: u32,
        services: ServicesList,
        timestamp: std::time::SystemTime,
        addr_recv: NetAddr,
        addr_sent: NetAddr,
        nonce: u64,
        agent: String,
        start_height: u32,
        relay: bool
    ) -> VersionMessage {
        Self {
            version,
            services,
            timestamp,
            addr_recv,
            addr_sent,
            nonce,
            agent,
            start_height,
            relay
        }
    }
}

#[derive(Debug, Clone)]
/// Verack message struct.
//  The verack message has no payload, it consists only of the header with the command string.
pub struct VerackMessage();

impl VerackMessage {
    pub fn new() -> VerackMessage {
        VerackMessage()
    }
}