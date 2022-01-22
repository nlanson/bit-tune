// network.rs
//
// Structures related to networking messages (ie NetAddr, VersionMessage, Verack) are
// implemented in this module.
//

use crate::{
    net::peer::{
        Peer,
        Port
    },
    msg::header::{
        Checksum,
        sha256d
    },
    encode::{
        Encode,
        Error
    }
};
use std::net::Ipv4Addr;
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



#[derive(Clone, Debug, PartialEq, Eq)]
/// When a network address is needed somewhere, this structure is used.
/// When not used in the version message, a time stamp is needed. (unimplented)
pub struct NetAddr {
    pub service: ServicesList,
    pub ip: Ipv4Addr,
    pub port: Port,
}

impl NetAddr {
    pub fn new(
        service: ServicesList,
        ip: Ipv4Addr,
        port: Port
    ) -> Self {
        Self {
            service,
            ip: ip,
            port: port
        }
    }
}

impl Default for NetAddr {
    fn default() -> NetAddr {
        Self {
            service: ServicesList::default(),
            ip: Ipv4Addr::new(0, 0, 0, 0),
            port: Port::from(0)
        }
    }
}

impl From<(Ipv4Addr, Port)> for NetAddr {
    fn from(net_info: (Ipv4Addr, Port)) -> NetAddr {
        Self {
            service: ServicesList::new(),
            ip: net_info.0,
            port: net_info.1
        }
    }
}


#[derive(Debug, Clone, Eq)]
/// The message payload for version commands.
pub struct VersionMessage {
    pub version: u32,
    pub service: ServicesList,
    pub timestamp: Duration,
    pub addr_recv: NetAddr,
    pub addr_from: NetAddr,
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
        addr_recv: NetAddr,
        addr_from: NetAddr,
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

impl Checksum for VersionMessage {
    fn checksum(&self) -> [u8; 4] {
        let mut ret: [u8; 4] = [0; 4];
        let mut payload = Vec::new();
        self.net_encode(&mut payload);

        ret.copy_from_slice(&sha256d(payload)[..4]);
        ret
    }
}

impl From<&Peer> for VersionMessage {
    /// Create a default VersionMessage struct from a peer with:
    /// * Protocol version 70016 (line 12: https://github.com/bitcoin/bitcoin/blob/master/src/version.h)
    /// * No service flags
    /// * Current time at fuction evoke
    /// * Default net address structs
    /// * Random nonce capped at u64 ceiling
    /// * Agent "bit-tune-v0.0.1"
    /// * Relay flag set to false
    fn from(peer: &Peer) -> Self {
        VersionMessage::new(
            70015, 
            ServicesList::default(), 
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("Failed to get time"), 
            NetAddr::new(ServicesList::default(), peer.addr, peer.port),
            NetAddr::default(),
            rand::thread_rng().gen_range(0..u64::MAX), 
            String::from("bit-tune-v0.0.1"), 
            0u32,
            false
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

#[derive(Debug, Clone)]
/// Verack message struct.
//  The verack message has no payload, it consists only of the header with the command string.
pub struct VerackMessage();

impl VerackMessage {
    pub fn new() -> VerackMessage {
        VerackMessage()
    }
}

impl Default for VerackMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl Checksum for VerackMessage {
    fn checksum(&self) -> [u8; 4] {
        // Sha256d of nothing precomputed:
        [0x5D, 0xF6, 0xE0, 0xE2]
    }
}