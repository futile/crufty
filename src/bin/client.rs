extern crate crufty;

// use crufty::util::run_state_machine;
// use crufty::application::AppTransition;

// fn main() {
//     run_state_machine(AppTransition::Startup);
// }

use std::str::FromStr;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

// use crufty::net::client::ClientConnection;

// fn main() {
//     let mut conn = ClientConnection::new().expect("could not create client connection");

//     conn.start_connect(&SocketAddr::from_str("127.0.0.1:13625").unwrap());

//     loop {
//         if let Some(event) = conn.handle() {
//             println!("event: {:?}", event);
//         }

//         thread::sleep(Duration::from_millis(100));
//     }
// }

use crufty::net::udp::{UdpConnection};

use std::time::Instant;

fn main() {
    let mut conn = UdpConnection::new(&"127.0.0.1:12365".parse().unwrap(),
                                      &"127.0.0.1:12366".parse().unwrap(),
                                      Duration::from_secs(3));

    // interval at which we send messages
    let send_interval = Duration::from_secs(2);

    let mut event_buffer = Vec::new();

    loop {
        let msg = "Hello, Udp!".as_bytes();
        conn.send_bytes(&msg);
        println!("msg: {:?}", msg);

        conn.update(Instant::now() + send_interval, &mut event_buffer);

        println!("{:?}", event_buffer);
    }
}
