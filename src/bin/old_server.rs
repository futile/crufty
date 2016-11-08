extern crate crufty;

use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use crufty::net::server::ServerBind;

fn main() {
    let mut bind = ServerBind::new(&SocketAddr::from_str("127.0.0.1:13625").unwrap())
        .expect("could not create ServerBind");

    loop {
        if let Some(event) = bind.handle() {
            println!("event: {:?}", event);
        }

        thread::sleep(Duration::from_millis(100));
    }
}
