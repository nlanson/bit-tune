// network.rs
//
// Structures related to networking messages (ie NetAddr, VersionMessage, Verack) are
// implemented in this module.
//

use crate::{
    encode::Error,
    address::Address
};
use std::collections::HashSet;
use std::time::{
    SystemTime,
    Duration
};
use rand::Rng;


#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
#[allow(dead_code)]
/// Node service flag to indicate what service are available on a node.
pub enum Service {
    None,
    Network,
    GetUTXO,
    Bloom,
    Witness,
    CompactFilters,
    NetworkLimited
}

// Constant array containing the right shift amount for each service flag.
pub const SERVICE_BITS: [usize; 6] = [
    0, // Network
    1, // GetUTXO
    2, // Bloom
    3, // Witness
    6, // CompactFilters
    10 // NetworkLimited
];

impl Service {
    pub fn value(&self) -> u64 {
        match self {
            // Each service is a bit flag
            Self::None => 0,                              // No service available
            Self::Network =>        (1<<SERVICE_BITS[0]), // Full chain history available
            Self::GetUTXO =>        (1<<SERVICE_BITS[1]), // Can be queried for UTXOs
            Self::Bloom =>          (1<<SERVICE_BITS[2]), // Capable of handling bloom filtered connections
            Self::Witness =>        (1<<SERVICE_BITS[3]), // Witness data available
            Self::CompactFilters => (1<<SERVICE_BITS[4]), // Can serve basic block filte requests
            Self::NetworkLimited => (1<<SERVICE_BITS[5])  // Can serve blocks from the last 2 days
        }
    }

    pub fn try_from_bit(flag: u64) -> Result<Self, Error> {
        match flag {
            0 => Ok(Self::None),
            1 => Ok(Self::Network),
            2 => Ok(Self::GetUTXO),
            4 => Ok(Self::Bloom),
            8 => Ok(Self::Witness),
            64 => Ok(Self::CompactFilters),
            1024 => Ok(Self::NetworkLimited),
            _ => Err(Error::InvalidData)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A list of service flags in a hash set.
/// DOES NOT ENFORCE CONFLICTING FLAGS
pub struct ServicesList(std::collections::HashSet<Service>);

impl ServicesList {
    pub fn new() -> Self {
        ServicesList(HashSet::new())
    }

    pub fn add_flag(&mut self, flag: Service) {
        self.0.insert(flag);
    }

    pub fn get_flags(&self) -> Vec<Service> {
        self.0.iter().map(|flag| *flag).collect()
    }
}

impl Default for ServicesList {
    fn default() -> Self {
        let mut flags = Self::new();
        flags.add_flag(Service::None);
        flags
    }
}

#[derive(Debug, Clone, Eq)]
/// The message payload for version commands.
pub struct VersionMessage {
    pub version: u32,
    pub service: ServicesList,
    pub timestamp: Duration,
    pub addr_recv: NetAddress,
    pub addr_from: NetAddress,
    pub nonce: u64,
    pub agent: String,
    pub start_height: u32,
    pub relay: bool
}

impl VersionMessage {
    pub fn new(
        version: u32,
        service: ServicesList,
        timestamp: Duration,
        addr_recv: NetAddress,
        addr_from: NetAddress,
        nonce: u64,
        agent: String,
        start_height: u32,
        relay: bool
    ) -> VersionMessage {
        Self {
            version,
            service,
            timestamp,
            addr_recv,
            addr_from,
            nonce,
            agent,
            start_height,
            relay
        }
    }
}

impl From<Address> for VersionMessage {
    /// Create a default VersionMessage struct from a peer with:
    /// * Protocol version 70016 (line 12: https://github.com/bitcoin/bitcoin/blob/master/src/version.h)
    /// * No service flags
    /// * Current time at fuction evoke
    /// * Default net address structs
    /// * Random nonce capped at u64 ceiling
    /// * Agent "bit-tune-v0.0.1"
    /// * Relay flag set to false
    fn from(address: Address) -> Self {
        VersionMessage::new(
            70015, 
            ServicesList::default(), 
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Failed to get time"), 
            NetAddress::new(ServicesList::default(), address),
            NetAddress::default(),
            rand::thread_rng().gen_range(0..u64::MAX), 
            String::from("bit-tune-v0.0.1"), 
            0u32,
            false // Setting this option to true will get the other node to broadcast transaction regardless of bloom filter status
        )
    }
}

impl PartialEq for VersionMessage {
    fn eq(&self, other: &Self) -> bool { 
        self.version == other.version &&
        self.service == other.service &&
        self.timestamp.as_secs() == other.timestamp.as_secs() && // Need to call as_secs() or else the code will be comparing floating points
        self.addr_from == other.addr_from &&
        self.addr_recv == other.addr_recv &&
        self.nonce == other.nonce &&
        self.agent == other.agent &&
        self.start_height == other.start_height &&
        self.relay == other.relay
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Data structure to pass around network addresses and related meta data in the bitcoin network
pub struct NetAddress {
    pub services: ServicesList,
    pub address: Address
}

impl NetAddress {
    pub fn new(services: ServicesList, address: Address) -> Self {
        Self {
            services,
            address
        }
    }
}

impl Default for NetAddress {
    fn default() -> Self {
        Self {
            services: ServicesList::default(),
            address: Address::me()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// NetAddress structure with a timestamp.
pub struct TimestampedNetAddress {
    pub timestamp: Duration,
    pub netaddress: NetAddress
}

impl TimestampedNetAddress {
    pub fn new(timestamp: Duration, netaddress: NetAddress) -> Self {
        Self {
            timestamp,
            netaddress
        }
    }
}

impl From<TimestampedNetAddress> for NetAddress {
    fn from(tsna: TimestampedNetAddress) -> Self {
        tsna.netaddress
    }
}