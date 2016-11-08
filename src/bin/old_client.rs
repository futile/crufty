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

use crufty::net::client::ClientConnection;

fn main() {
    let mut conn = ClientConnection::new().expect("could not create client connection");

    conn.start_connect(&SocketAddr::from_str("127.0.0.1:13625").unwrap());

    loop {
        if let Some(event) = conn.handle() {
            println!("event: {:?}", event);
        }

        thread::sleep(Duration::from_millis(100));
    }
}
