// encode.rs
//
// Module implementing the encoding/decoding of encodable structures
//

use std::net::Ipv4Addr;

use crate::{
    msg::{
        data::{
            Message,
            MessagePayload
        },
        header::{
            VariableInteger,
            Magic,
            Command,
            MessageHeader
        },
        network::{
            NetAddr,
            ServicesList,
            VersionMessage,
            VerackMessage
        }
    },
    net::peer::{
        Port
    }
};

/// Trait to encode self into a format acceptable by the Bitcoin P2P network.
pub trait Encode {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write;
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
integer_le_encode!(u8);
integer_le_encode!(u16);
integer_le_encode!(u32);
integer_le_encode!(u64);
integer_le_encode!(usize);

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
array_encode!(4);
array_encode!(2);


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

impl Encode for Magic {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        self.bytes().net_encode(w)
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

impl Encode for MessageHeader {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.magic.net_encode(&mut w) +
        self.command.net_encode(&mut w) +
        self.length.net_encode(&mut w) +
        self.checksum.net_encode(&mut w)
    }
}

impl Encode for Message {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.header.net_encode(&mut w) +
        self.payload.net_encode(&mut w)
    }
}

impl Encode for MessagePayload {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        match self {
            MessagePayload::Version(v) => v.net_encode(w),
            MessagePayload::Verack(v) => v.net_encode(w)
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

impl Encode for Port {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        self.0.net_encode(w)
    }
}

impl Encode for Ipv4Addr {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        // Ipv4 addresses are encoded as an Ipv4 mapped Ipv6 address.
        w.write(&self.to_ipv6_mapped().octets()).expect("Failed to write")
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

impl Encode for NetAddr {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.services.net_encode(&mut w) +
        self.ip.net_encode(&mut w) +
        self.port.net_encode(&mut w)
    }
}

impl Encode for std::time::SystemTime {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write {
        self
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Could not get unix time")
            .as_secs()
            .net_encode(w)
    }
}

impl Encode for VersionMessage {
    fn net_encode<W>(&self, mut w: W) -> usize
    where W: std::io::Write {
        self.version.net_encode(&mut w) +
        self.services.net_encode(&mut w) +
        self.timestamp.net_encode(&mut w) +
        self.addr_recv.net_encode(&mut w) +
        self.addr_sent.net_encode(&mut w) +
        self.nonce.net_encode(&mut w) +
        self.agent.net_encode(&mut w) +
        self.start_height.net_encode(&mut w) +
        (self.relay as u8).net_encode(&mut w)
    }
}

impl Encode for VerackMessage {
    fn net_encode<W>(&self, _w: W) -> usize
    where W: std::io::Write {
        0
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::network::Services;

    #[test]
    fn varint_test() {
        let ints: [u64; 9] = [0x01, 0xFC, 0xFD, 0x1000, 0xFFFF, 0x10000, 0x55555, 0xFFFF_FFFF, 0x1000_0000_0000];
        let lens: [usize; 9] = [1, 1, 3, 3, 3, 5, 5, 5, 9];

        for i in 0..ints.len() {
            assert_eq!(VariableInteger::from(ints[i]).net_encode(Vec::new()), lens[i])
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
        flags.add_flag(Services::Network);
        
        let mut encoded = Vec::new();
        flags.net_encode(&mut encoded);
        
        assert_eq!(encoded, &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }
}