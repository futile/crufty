extern crate crufty;

use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::time::{Instant, Duration};

// use crufty::net::server::ServerBind;

// fn main() {
//     let mut bind = ServerBind::new(&SocketAddr::from_str("127.0.0.1:13625").unwrap())
//         .expect("could not create ServerBind");

//     loop {
//         if let Some(event) = bind.handle() {
//             println!("event: {:?}", event);
//         }

//         thread::sleep(Duration::from_millis(100));
//     }
// }

use crufty::net::udp::{UdpConnection, UdpConnectionEvent};

fn main() {
    let mut conn = UdpConnection::new(&"127.0.0.1:12366".parse().unwrap(),
                                      &"127.0.0.1:12365".parse().unwrap(),
                                      Duration::from_secs(3));

    let mut event_buffer = vec![];

    loop {
        conn.update(Instant::now() + Duration::from_secs(3), &mut event_buffer);

        for event in event_buffer.drain(..) {
            match event {
                UdpConnectionEvent::MessageReceived(msg) => {
                    println!("msg raw: {:?}", msg);
                    let msg_str = ::std::str::from_utf8(&msg).unwrap();
                    println!("Message: {}", msg_str);

                    conn.send_bytes(&format!("Ping: '{}'", msg_str).as_bytes());
                }
                mto @ UdpConnectionEvent::MessageTimedOut(_) => println!("{:?}", mto),
            }
        }

        println!("{:?}", event_buffer);
    }
}
