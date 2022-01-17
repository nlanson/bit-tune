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
//  - Payload hashing for message header checksum
//  - Convinience function for creating messages that will be used
//  - TCP Streams


// Modules
mod seeds;
mod net;
mod netmsg;
mod netmsgheader;
mod encode;

use net::*;
use netmsg::{
    VersionMessage,
    NetAddr,
    ServicesList,
    Services
};

use crate::encode::Encode;

fn main() {
    // Currently, running the program will simply try and connect to the minimum specified
    // number of peers, and then spit out a version message to be sent out to the first peer.
    let args = ApplicationArgs::from(std::env::args());

    let peers = Peer::get(args.min_peers);
        
    println!("Got {} peers...", peers.len());

    // Create version message payload (No headers yet...)
    let version = 70015; // Current protocol version
    let mut services = ServicesList::new(); 
    services.add_flag(Services::None); // No services
    let timestamp = std::time::SystemTime::now(); // Current time
    let addr_recv = NetAddr::new(services.clone(), peers[0].addr, peers[0].port); // Receiving address. Services should not be none
    let addr_sent = NetAddr::default(); // Sending address (our local IP)
    let nonce = 16735069437859780935u64; // Random nonce (Should use RNG)
    let agent = String::from("cmdline:1"); // Agent (Can be anything really)
    let start_height = 0u32; // Start height
    let relay = false; // Relay

    let version_message = VersionMessage::new(
        version, 
        services, 
        timestamp, 
        addr_recv, 
        addr_sent,
        nonce, 
        agent, 
        start_height,
        relay
    );

    let mut msg = Vec::new();
    version_message.net_encode(&mut msg);
    println!("{:02x?}", msg);
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