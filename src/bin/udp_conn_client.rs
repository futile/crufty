extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{UdpConnection};

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
        event_buffer.clear();
    }
}
