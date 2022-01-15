// encode.rs
//
// Module implementing the encoding/decoding of encodable structures
//

use crate::{
    netmsg::{
        VariableInteger,
        Magic,
        Command
    }
};

/// Trait to encode self into a format acceptable by the Bitcoin P2P network.
pub trait Encode {
    fn net_encode<W>(&self, w: W) -> usize
    where W: std::io::Write;
}

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

// Most integers are encoded as little endian
integer_le_encode!(u8);
integer_le_encode!(u16);
integer_le_encode!(u32);
integer_le_encode!(u64);


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


#[cfg(test)]
mod tests {
    use super::*;

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
}