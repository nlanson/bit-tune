// encode.rs
//
// Module implementing the encoding/decoding of encodable structures
//

use std::net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr
};
use std::time::Duration;

use crate::{
    msg::{
        data::{
            Message,
            MessagePayload,
            EmptyPayload
        },
        header::{
            Magic,
            Command,
            MessageHeader
        },
        network::{
            ServicesList,
            VersionMessage,
            Service,
            SERVICE_BITS,
            NetAddress,
            TimestampedNetAddress
        },
        inventory::{
            Inventory
        }
    },
    blockdata::{
        VariableInteger
    },
    address::Address
};

use crate::bitcoin::{
    hash_types::{
        Txid,
        BlockHash
    },
    consensus::{
        Encodable,
        Decodable
    }
};

/// Trait to encode self into a format acceptable by the Bitcoin P2P network.
pub trait Encode {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write;
}

pub trait Decode: Sized {
    fn net_decode<R>(r: R) -> Result<Self, Error>
    where R: std::io::Read;
}

#[derive(Debug)]
pub enum Error {
    InvalidData,
    BadNetworkMagic(Magic),
    Io(std::io::Error),
    UnknownCommand(String)
}

/// Macro to encode integers in little endian.
macro_rules! integer_le_encode {
    ($int: ty) => {
        impl Encode for $int {
            fn net_encode<W>(&self, mut w: W) -> usize
            where W: std::io::Write {
                w.write(&self.to_le_bytes()).expect("Failed to write")
            }
        }
    };
}

/// Macro to decode little endian integers
macro_rules! integer_le_decode {
    ($int: ty) => {
        impl Decode for $int {
            fn net_decode<R>(mut r: R) -> Result<Self, Error>
            where
                R: std::io::Read ,
                Self: Sized
            {
                let mut buf = [0; std::mem::size_of::<$int>()];
                r.read_exact(&mut buf).expect("Failed to read");
                
                let mut ret: u64 = 0;
                let mut i = buf.len() - 1;
                loop {
                    ret ^= buf[i] as u64;
                    if i == 0 { break }
                    i-=1;
                    ret = ret << 8; 
                }
                
                Ok(ret as $int)
            }
        }
    }
}

integer_le_encode!(u8);
integer_le_encode!(u16);
integer_le_encode!(u32);
integer_le_encode!(u64);
integer_le_encode!(usize);

integer_le_decode!(u8);
integer_le_decode!(u16);
integer_le_decode!(u32);
integer_le_decode!(u64);
integer_le_decode!(usize);


/// Macro to encode arrays
macro_rules! array_encode {
    ($len: expr) => {
        impl Encode for [u8; $len] {
            fn net_encode<W>(&self, mut w: W) -> usize
            where W: std::io::Write {
                w.write(self).expect("Failed to write")
            }
        }
    };
}

macro_rules! array_decode {
    ($len: expr) => {
        impl Decode for [u8; $len] {
            fn net_decode<R>(mut r: R) -> Result<Self, Error>
            where
                R: std::io::Read ,
                Self: Sized
            {
                let mut buf: [u8; $len] = [0; $len];
                r.read_exact(&mut buf).expect("Failed to read");
                
                Ok(buf)
            }
        }
    }
}

array_encode!(4);
array_encode!(2);
array_encode!(16);
array_encode!(32);

array_decode!(4);
array_decode!(2);
array_decode!(16);
array_decode!(32);


/// Encode a vector of elements that implement the Encode trait.
impl<T: Encode> Encode for Vec<T> {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        let mut size: usize = 0;
        for elem in self {
            size += elem.net_encode(&mut w)
        }
        size
    }
}


impl Encode for VariableInteger {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        match self.0 {
            0..=0xFC => {
                (self.0 as u8).net_encode(w)
            },
            0xFD..=0xFFFF => {
                w.write(&[0xFD]).expect("Failed to write");
                (self.0 as u16).net_encode(w);
                3
            },
            0x10000..=0xFFFF_FFFF => {
                w.write(&[0xFE]).expect("Failed to write");
                (self.0 as u32).net_encode(w);
                5
            },
            _ => {
                w.write(&[0xFF]).expect("Failed to write");
                (self.0 as u64).net_encode(w);
                9
            }
        }
    }
}

impl Decode for VariableInteger {
    fn net_decode<R: std::io::Read >(mut r: R) -> Result<Self, Error> {
        // Read the first byte as a length indicator and match it with protocol varint length indicators
        // to set the buffer length of the integer that follows
        let mut len_indic: [u8; 1] = [0; 1];
        r.read_exact(&mut len_indic).expect("Failed to read");
        let mut buf: Vec<u8> = match len_indic[0] {
            0xFD => vec![0; 2],
            0xFE => vec![0; 4],
            0xFF => vec![0; 8],
            _ =>    vec![0; 1]
        };

        // The varint did not have a prefix, return the value
        if buf.len() == 1 {
            return Ok(VariableInteger::from(len_indic[0] as u64))
        }

        // The varint did have a length indicating prefix.
        // Read the integer and append zeroes to cast it as a LE u64.
        r.read_exact(&mut buf).expect("Failed to read");
        while buf.len() != 8 {
            buf.push(0x00);
        }

        // Return the LE u64 decoded as a Varint
        return Ok(VariableInteger::from(u64::net_decode(&buf[..])?))
    }
}

impl Encode for Magic {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        self.bytes().net_encode(w)
    }
}

impl Decode for Magic {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let mut buf = [0; 4];
        r.read(&mut buf).expect("Failed to read");
        buf.reverse();

        // If the network magic is not known, return an error.
        match Magic::from(buf) {
            Magic::Unknown(v) => return Err(Error::BadNetworkMagic(Magic::Unknown(v))),
            x => Ok(x)
        }
    }
}

impl Encode for Command {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        let mut buf: [u8; 12] = [0; 12];
        let cmd_str = self.to_str().as_bytes();
        buf[..cmd_str.len()].copy_from_slice(&cmd_str);
        w.write(&buf).expect("Failed to write")
    }
}

impl Decode for Command {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let mut buf = [0; 12];
        r.read(&mut buf).expect("Failed to read");

        Self::from_str(
        buf
                .iter()
                .take_while(|x| **x != 0x00)
                .map(|c| *c as char)
                .collect::<String>()
        )
    }
}

impl Encode for MessageHeader {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.magic.net_encode(&mut w) +
        self.command.net_encode(&mut w) +
        self.length.net_encode(&mut w) +
        self.checksum.net_encode(&mut w)
    }
}

impl Decode for MessageHeader {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let magic = Magic::net_decode(&mut r)?;
        let command =  match Command::net_decode(&mut r) {
            Ok(x) => x,
            Err(err) => match err {
                Error::UnknownCommand(x) => Command::Unknown(x),
                x => return Err(x)
            }
        };
        let length: u32 = Decode::net_decode(&mut r)?;
        let checksum: [u8; 4] = Decode::net_decode(&mut r)?;

        Ok(
            Self::new(magic, command, length as usize, checksum)
        )
    }
}

impl Encode for Message {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        // If the payload is empty, check that the heder has zero as the length.
        if self.payload == MessagePayload::EmptyPayload {
            assert_eq!(self.header.length, 0)
        }
        
        self.header.net_encode(&mut w) +
        self.payload.net_encode(&mut w)
    }
}

impl Decode for Message {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let header: MessageHeader = Decode::net_decode(&mut r)?;

        // Message payload doesn't implement the [`Decode`] trait on it's own as
        // it cannot be decoded without the header context
        let payload: MessagePayload = match header.command {
            Command::Version => MessagePayload::Version(Decode::net_decode(&mut r)?),
            Command::Verack => MessagePayload::EmptyPayload,
            Command::SendHeaders => MessagePayload::EmptyPayload,
            Command::WTxIdRelay => MessagePayload::EmptyPayload,
            Command::Ping => MessagePayload::PingPong(Decode::net_decode(&mut r)?),
            Command::Pong => MessagePayload::PingPong(Decode::net_decode(&mut r)?),
            Command::Addr => { 
                let count: VariableInteger = Decode::net_decode(&mut r)?;
                assert!(count.inner() <= 100); // Max of 100 addresses
                let mut addrs: Vec<TimestampedNetAddress> = Vec::new();
                for _ in 0..count.inner() {
                    addrs.push(Decode::net_decode(&mut r)?)
                }
                MessagePayload::AddrList(addrs)
            },
            Command::GetAddr => MessagePayload::EmptyPayload,
            Command::Inv => {
                let count: VariableInteger = Decode::net_decode(&mut r)?;
                let mut inv_items: Vec<Inventory> = Vec::new();
                for _ in 0..count.inner() {
                    inv_items.push(Decode::net_decode(&mut r)?)
                }

                MessagePayload::InvVect(inv_items)
            }

            // Upon receiving an unknown/invalid command in the header...
            Command::Unknown(_) => {
                // Consume the payload and store it as a hex dump
                let mut buf = vec![0; header.length as usize];
                r.read_exact(&mut buf).expect("Failed to read");

                MessagePayload::Dump(buf)
            }
        };
        
        Ok(
            Message {
                header,
                payload
            }
        )
    }
}

impl Encode for MessagePayload {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        match self {
            MessagePayload::Version(v) => v.net_encode(w),
            MessagePayload::PingPong(int) => int.net_encode(w),
            MessagePayload::EmptyPayload =>  EmptyPayload.net_encode(w),
            MessagePayload::AddrList(addrs) => VariableInteger::from(addrs.len()).net_encode(&mut w) + addrs.net_encode(&mut w),
            MessagePayload::InvVect(inv) => VariableInteger::from(inv.len()).net_encode(&mut w) + inv.net_encode(&mut w),
            MessagePayload::Dump(d) => d.net_encode(w)
        }
    }
}

/// Strings are encoded as var string which is the string bytes with a varint prefixed
impl Encode for String {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        VariableInteger::from(self.len()).net_encode(&mut w) +
        w.write(self.as_bytes()).expect("Failed to write")
    }
}

impl Decode for String {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let varint: VariableInteger = Decode::net_decode(&mut r)?;
        let mut buf = vec![0; varint.inner() as usize];
        r.read_exact(&mut buf).expect("Failed to read");

        Ok(
            buf
                .iter()
                .map(|x| *x as char)
                .collect::<String>()
        )
    }
}

impl Encode for Address {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.0.net_encode(&mut w)
    }
}

impl Decode for Address {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        Ok(Self(Decode::net_decode(&mut r)?))
    }
}

impl Encode for SocketAddr {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.ip().net_encode(&mut w) +
        self.port().to_be_bytes().net_encode(&mut w)
    }
}

impl Decode for SocketAddr {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let ip = Decode::net_decode(&mut r)?;
        let portb: [u8; 2] = Decode::net_decode(&mut r)?;
        Ok(SocketAddr::new(ip, portb[0] as u16 >> 8 | portb[1] as u16))
    }
}

impl Encode for IpAddr {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        match self {
            Self::V4(ip) => ip.net_encode(&mut w),
            Self::V6(ip) => ip.net_encode(&mut w)
        }
    }
}

impl Decode for IpAddr {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        // Decode as ipv6...
        let ipv6: Ipv6Addr = Decode::net_decode(&mut r)?;

        // attempt to convert to v4...
        match ipv6.to_ipv4() {
            Some(ipv4) => Ok(IpAddr::V4(ipv4)),
            None => Ok(IpAddr::V6(ipv6))
        }
    }
}

impl Encode for Ipv4Addr {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        // Ipv4 addresses are encoded as an Ipv4 mapped Ipv6 address.
        self
            .to_ipv6_mapped()
            .net_encode(&mut w)
    }
}

impl Decode for Ipv4Addr {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        // Ipv4 addresses are encoded as IPv6 mapped addresses.
        let mut ipv4b: [u8; 4] = [0; 4];
        let ipv6b: [u8; 16] = Decode::net_decode(&mut r)?;
        ipv4b.copy_from_slice(&ipv6b[ipv6b.len()-4..]);
        Ok(Ipv4Addr::from(ipv4b))
    }
}

impl Encode for Ipv6Addr {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self
            .octets()
            .net_encode(&mut w)
    }
}

impl Decode for Ipv6Addr {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let bytes: [u8; 16] = Decode::net_decode(&mut r)?;
        Ok(Ipv6Addr::from(bytes))
    }
}

impl Encode for ServicesList {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        // Collect all the service flags and XOR them up
        let flag: u64 = 
        self
            .get_flags()
            .iter()
            .fold(
                0,
                |acc, num| 
                acc ^ num.value()
            );

        flag.net_encode(w) //always 8 bytes
    }
}

impl Decode for ServicesList {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let flags: u64 = Decode::net_decode(&mut r)?;

        // Early exit for flags with no bits set...
        if flags == 0 {
            return Ok(ServicesList::default());
        }

        // Iterate through all possible set flags and record any flags that are set.
        let mut services = ServicesList::new();
        for bitp in SERVICE_BITS {
            if flags & (1<<bitp) == (1<<bitp) {
                let flag = Service::try_from_bit(1<<bitp)?;
                services.add_flag(flag);
            }
        }

        Ok(services)
    }
}

impl Encode for NetAddress {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.services.net_encode(&mut w) +
        self.address.net_encode(&mut w)
    }
}

impl Decode for NetAddress {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        Ok(
            Self::new(
                Decode::net_decode(&mut r)?,
                Decode::net_decode(&mut r)?
            )
        )
    }
}

impl Encode for TimestampedNetAddress {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        // Timestamp in netaddr is encoded as a 32bit int
        (self.timestamp.as_secs() as u32).net_encode(&mut w) +
        self.netaddress.net_encode(&mut w)
    }
}

impl Decode for TimestampedNetAddress {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let secs: u32 = Decode::net_decode(&mut r)?;
        Ok(
            Self::new(
                Duration::from_secs(secs as u64),
                Decode::net_decode(&mut r)?
            )
        )
    }
}

impl Encode for Duration {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        // Duration is encoded as a 64 bit integer
        self
            .as_secs()
            .net_encode(w)
    }
}

impl Decode for Duration {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        // Decode a duration where the duration is encoded as a 64bit integer
        Ok(Duration::from_secs(Decode::net_decode(&mut r)?))
    }
}

impl Encode for VersionMessage {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.version.net_encode(&mut w) +
        self.service.net_encode(&mut w) +
        self.timestamp.net_encode(&mut w) +
        self.addr_recv.net_encode(&mut w) +
        self.addr_from.net_encode(&mut w) +
        self.nonce.net_encode(&mut w) +
        self.agent.net_encode(&mut w) +
        self.start_height.net_encode(&mut w) +
        (self.relay as u8).net_encode(&mut w)
    }
}

impl Decode for VersionMessage {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        let version: u32 = Decode::net_decode(&mut r)?;
        let services: ServicesList = Decode::net_decode(&mut r)?;
        let timestamp: Duration = Decode::net_decode(&mut r)?;
        let addr_recv: NetAddress = Decode::net_decode(&mut r)?;
        let addr_from: NetAddress = Decode::net_decode(&mut r)?;
        let nonce: u64 = Decode::net_decode(&mut r)?;
        let agent: String = Decode::net_decode(&mut r)?;
        let start_height: u32 = Decode::net_decode(&mut r)?;
        let relay = match u8::net_decode(&mut r)? {
            0 => false,
            _ => true
        };
        
        
        Ok(VersionMessage::new(
            version,
            services,
            timestamp,
            addr_recv,
            addr_from,
            nonce,
            agent,
            start_height,
            relay
        ))
    }
}

impl Encode for EmptyPayload {
    fn net_encode<W>(&self, _w: W) -> usize
    where W: std::io::Write {
        0
    }
}

impl Decode for EmptyPayload {
    fn net_decode<R>(_r: R) -> Result<Self, Error>
    where R: std::io::Read {
        Ok(Self::default())
    }
}


impl Encode for Inventory {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.identifier().net_encode(&mut w) +
        match self {
            Self::Error => [0; 32],
            Self::Tx(_) => self.inner(),
            Self::Block(_) => self.inner(),
            Self::FilteredBlock(_) => self.inner(),
            Self::CompactBlock(_) => self.inner(),
            Self::WitnessTx(_) => self.inner(),
            Self::WitnessBlock(_) => self.inner(),
            Self::FilteredWitnessBlock(_) => self.inner(),
            Self::Unknown{inv_type: _, hash: h} => *h
        }.net_encode(&mut w)
    }
}

impl Decode for Inventory {
    fn net_decode<R>(mut r: R) -> Result<Self, Error>
    where R: std::io::Read {
        Ok(
            Self::from_id_and_hash(
                Decode::net_decode(&mut r)?,
                Decode::net_decode(&mut r)?
            )
        )
    }
}

// Macro to implement hashing for the imported hash types from rust-bitcoin
macro_rules! bitcoin_hash_encode {
    ($hash: ty) => {
        impl Encode for $hash {
            fn net_encode<W>(&self, mut w: W) -> usize
            where W: std::io::Write {
                self.consensus_encode(&mut w).expect("Failed to write")
            }
        }

        impl Decode for $hash {
            fn net_decode<R>(mut r: R) -> Result<Self, Error>
            where R: std::io::Read {
                Ok(Self::consensus_decode(&mut r)?)
            }
        }
    };
}

// Conversion of encode::Error to Error
impl From<crate::bitcoin::consensus::encode::Error> for Error {
    fn from(err: crate::bitcoin::consensus::encode::Error) -> Error {
        match err {
            crate::bitcoin::consensus::encode::Error::Io(x) => Error::Io(x),
            _ => Error::InvalidData
        }
    }
}

// Implement encoding for Txid and Blockhashes imported from rust-bitcoin
bitcoin_hash_encode!(Txid);
bitcoin_hash_encode!(BlockHash);








#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::network::Service;

    #[test]
    fn varint_test() {
        let ints: [u64; 9] = [0x01, 0xFC, 0xFD, 0x1000, 0xFFFF, 0x10000, 0x55555, 0xFFFF_FFFF, 0x1000_0000_0000];
        let lens: [usize; 9] = [1, 1, 3, 3, 3, 5, 5, 5, 9];

        for i in 0..ints.len() {
            let mut enc: Vec<u8> = Vec::new();
            assert_eq!(VariableInteger::from(ints[i]).net_encode(&mut enc), lens[i]);
            assert_eq!(VariableInteger::net_decode(&enc[..]).unwrap(), VariableInteger::from(ints[i]))
        }
    }

    #[test]
    fn network_magic() {
        let mut main: Vec<u8> = Vec::new();
        let mut test: Vec<u8> = Vec::new();

        Magic::Main.net_encode(&mut main);
        Magic::Test.net_encode(&mut test);

        assert_eq!(main, [0xF9, 0xBE, 0xB4, 0xD9]);
        assert_eq!(test, [0xFA, 0xBF, 0xB5, 0xDA]);
    }

    #[test]
    fn service_flags() {
        let mut flags = ServicesList::new();
        flags.add_flag(Service::Network);
        
        let mut encoded = Vec::new();
        flags.net_encode(&mut encoded);
        
        assert_eq!(encoded, &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    #[test]
    fn integer_le() {
        let int: u8 = 0xFF;
        let mut enc: Vec<u8> = Vec::new();
        int.net_encode(&mut enc);
        let dec = u8::net_decode(&enc[..]).expect("Failed to decode");
        assert_eq!(int, dec);

        let int: u16 = 0xFFFF;
        let mut enc: Vec<u8> = Vec::new();
        int.net_encode(&mut enc);
        let dec = u16::net_decode(&enc[..]).expect("Failed to decode");
        assert_eq!(int, dec);

        let int: u32 = 0xFFFF_FFFF;
        let mut enc: Vec<u8> = Vec::new();
        int.net_encode(&mut enc);
        let dec = u32::net_decode(&enc[..]).expect("Failed to decode");
        assert_eq!(int, dec);

        let int: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let mut enc: Vec<u8> = Vec::new();
        int.net_encode(&mut enc);
        let dec = u64::net_decode(&enc[..]).expect("Failed to decode");
        assert_eq!(int, dec);
    }

    #[test]
    fn header_decode() {
        let header = MessageHeader::new(Magic::Main, Command::Verack, 00, [0x5D, 0xF6, 0xE0, 0xE2]);
        let mut enc: Vec<u8> = Vec::new();
        header.net_encode(&mut enc);
        let dec: MessageHeader = Decode::net_decode(&enc[..]).expect("Failed to decode");
        assert_eq!(header, dec);
    }

    #[test]
    fn version_encode_decode() {
        let peer = crate::address::Address::me();
        let vm = VersionMessage::from(peer);
        let mut enc = Vec::new();
        vm.net_encode(&mut enc);
        let dec: VersionMessage = Decode::net_decode(&enc[..]).expect("Failed to decode");

        assert_eq!(vm, dec);
    }

    #[test]
    fn getaddr_encdec() {
        let msg = Message::new(MessagePayload::EmptyPayload, Magic::Main, Command::GetAddr);
        let mut enc = Vec::new();
        msg.net_encode(&mut enc);
        let dec: Message = Decode::net_decode(&enc[..]).expect("Failed to decode");

        assert_eq!(msg, dec);
    }
}