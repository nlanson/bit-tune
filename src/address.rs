use std::net::{
    SocketAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Structure representing an ip address + port combination
pub struct Address(pub SocketAddr);

impl Address {
    /// Get the local IP representation of this machine
    pub fn me() -> Self {
        Address(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
        )
    }

    /// Get the IP stored in self
    pub fn ip(&self) -> IpAddr {
        self.0.ip()
    }

    /// Get the port stoted in self
    pub fn port(&self) -> u16 {
        self.0.port()
    }

    /// Return the underlying SocketAddr structure
    pub fn inner(&self) -> SocketAddr {
        self.0
    }
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Self(addr)
    }
}