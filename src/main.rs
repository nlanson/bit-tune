// Bit tune radio
//
// Tune into chit-chat between computers of the Bitcoin P2P network 🧘
//
//
// General todos:
//  - Get simple message creation and transmission working.
//    The version message will be good for this.
//  - Decode incoming messages.
//
//  Immediate todos:
//  - TCP Streams. Multithreaded?


// Modules
mod seeds;
mod net;
mod msg;
mod encode;


pub use rand::Rng;
use net::{
    peer::*,
    stream::stream_from
};
use encode::Encode;
use msg::{
    network::{
        VersionMessage,
        VerackMessage
    },
    header::{
        Magic,
        Command
    },
    data::{
        Message,
        MessagePayload
    }
};

use std::{
    io::{
        Write,
        Read
    }
};

use crate::{encode::Decode, net::stream};


fn main() {
    // Program is not yet fully functional.
    // Currently the program can:
    //  - Detect and record working peers
    //  - Create "version" and "verack" messages
    //  - Open a TCP stream with working peers
    //  - Send "version" message to a peer and read the reply as a hex dump
    // 
    // Below shows examples of working aspects of the program:

    // Connect the minimum number of peers:
    let args = ApplicationArgs::from(std::env::args());
    let peers = Peer::get(args.min_peers).unwrap();
    println!("Got {} peers...", peers.len());


    // Create version message using the first available peer...
    let version_message = VersionMessage::from(&peers[0]);
    let payload = MessagePayload::from(version_message);
    let command = Command::from(&payload);
    let msg: Message = Message::new(payload, Magic::Main, command);
    let mut first_message: Vec<u8> = Vec::new();
    msg.net_encode(&mut first_message);


    // Create a Verack message
    let payload = MessagePayload::from(VerackMessage::new());
    let command = Command::from(&payload);
    let msg = Message::new(payload, Magic::Main, command);
    let mut second_message: Vec<u8> = Vec::new();
    msg.net_encode(&mut second_message);

    // Open a TCP stream with the first peer
    if let Ok(mut stream) = stream_from(peers[0]) {
        // Send a version message
        let _ = stream.write(&first_message);
        println!("Sent version message");

        let read_stream = stream.try_clone().unwrap();
        let mut stream_reader = std::io::BufReader::new(read_stream);
        
        // Loop and retrieve new messages
        loop {
            // Will fail here if an unknown/invalid message is received.
            // If a bad message is received here, the program will not be able to recover
            // as the buffer would be distrupted.
            let reply = Message::net_decode(&mut stream_reader).expect("Failed to decode");

            match reply.payload {
                MessagePayload::Version(_) => {
                    println!("Received version message");
                    let _ = stream.write(&second_message);
                    println!("Sent verack message");
                },
                MessagePayload::Verack(_) => {
                    println!("Received verack message");
                    break;
                }
            }
        }
        let _ = stream.shutdown(std::net::Shutdown::Both);
    } else {
        eprintln!("Failed to open connection");
    }
}




#[derive(Debug)]
struct ApplicationArgs {
    min_peers: usize  // Argument to specify the minimum number of peers to initially discover.
}

impl From<std::env::Args> for ApplicationArgs {
    fn from(args: std::env::Args) -> Self {
        let mut args = args.collect::<Vec<String>>();
        match args.len() {
            // If there is only 1 arg, use the defaults
            1 => Self::default(),

            // If there is more than 1 arg...
            _ => {
                args.remove(0); // remove the 0th arg (target directory)

                // Extract arguments
                let mut f_args = Self::default();
                for arg in args {
                    if arg.contains("min-peers") {
                        f_args.min_peers = arg
                            .split("=")
                            .collect::<Vec<&str>>()[1]
                            .parse::<usize>()
                            .expect("Invalid parameter for `min-peers`.")
                    }
                }
                
                // Validate arguments
                if f_args.min_peers == 0 { panic!("Minimum peers cannot be zero") }

                // Return
                f_args
            }
        }
    }
}

impl Default for ApplicationArgs {
    fn default() -> Self {
        Self {
            min_peers: 3
        }
    }
}