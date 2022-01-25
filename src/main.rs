// Bit tune radio
//
// Tune into chit-chat between computers of the Bitcoin P2P network 🧘
//
//
//  Todos:
//  - TCP Streams. Multithreaded to maintain multiple peers?
//  - Implement other common network messages
//  - Concurrent peer connections:
//    Pass off individual TCP streams to worker threads and get the main thread to synchronise data
//    from each peer connection using threadpools, MPSC, Mutex and etc concurrent data structures.
//    This will enable the program to connect to multiple nodes at once and maintain information in the main
//    thread (ie addr, inv and tx).
//  - Event loop for maintaining active peer connections through ping messages.
//  - Figure out how to receive new unverified TX commands (mempool?)...
//  - Other cool stuff...


// Modules
mod seeds;
mod net;
mod msg;
mod encode;

use std::io::Write;
use net::{
    peer::*,
    stream::stream_from
};
use encode::{Encode, Decode};
use msg::{
    network::{
        VersionMessage
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


fn main() {
    // Program is not yet fully functional.
    // Currently the program can:
    //  - Detect and working peers
    //  - Send "version" and "verack" messages to a peer and decode the reply if it is a known command
    //    (handshake)
    //  - Listen to a peer indefinitely
    //  - Reply to "ping" messages
    // 
    // Below shows examples of working aspects of the program:

    // Connect the minimum number of peers:
    let args = ApplicationArgs::from(std::env::args());
    let peers = Peer::get(args.min_peers).unwrap();
    println!("Got {} peers...", peers.len());


    // Create version message using the first available peer...
    let version_message = VersionMessage::from(&peers[0]);
    let payload = MessagePayload::from(version_message);
    let command = Command::Version;
    let msg: Message = Message::new(payload, Magic::Main, command);
    let mut first_message: Vec<u8> = Vec::new();
    msg.net_encode(&mut first_message);


    // Create a Verack message
    let payload = MessagePayload::EmptyPayload;
    let command = Command::Verack;
    let msg = Message::new(payload, Magic::Main, command);
    let mut second_message: Vec<u8> = Vec::new();
    msg.net_encode(&mut second_message);

    
    // Open a TCP stream with the first peer and send the version message...
    let mut stream = stream_from(peers[0]).expect("Failed to create stream");
    let _ = stream.write(&first_message);
    println!("Sent version message!");

    // Setup the stream reader with a buffer...
    let read_stream = stream.try_clone().expect("Failed to clone TCP stream");
    let mut stream_reader = std::io::BufReader::new(read_stream);

    // Listen for messages and break when a Verack message is received...
    loop {
        // If an unknown message is received, the program will panic here.
        let reply = Message::net_decode(&mut stream_reader).expect("Failed to decode");
        
        match reply.header.command {
            Command::Version => {
                println!("Received version message");
                let _ = stream.write(&second_message);
                println!("Sent verack message");
            },
            Command::Verack => {
                println!("Received verack message");
            },
            Command::Ping => {
                println!("Received ping message");
                let mut pong_msg = Vec::new();
                Message::new(reply.payload, Magic::Main, Command::Pong).net_encode(&mut pong_msg);
                let _ = stream.write(&mut pong_msg);
                println!("Send pong message");
            }
            cmd => println!("Received other message: {}", cmd.to_str())
        }
    }

    // let _ = stream.shutdown(std::net::Shutdown::Both);
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