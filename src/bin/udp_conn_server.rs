extern crate crufty;

use std::time::Duration;

use crufty::net::udp::{self, UdpConnection, ReceiveEvent};

fn main() {
    let mut conn = UdpConnection::new(&"127.0.0.1:12366".parse().unwrap(),
                                  &"127.0.0.1:12365".parse().unwrap(),
                                  Duration::from_secs(3));
    let mut to_send = None;
    let to_send = &mut to_send;
    loop {
        conn.recv_with_timeout(None, |e| match e {
            ReceiveEvent::NewAck(msg_id, rtt) => {
                println!("{:?} done, took {}ms", msg_id, udp::dur_to_ms(&rtt));
            }
            ReceiveEvent::NewData(data) => {
                *to_send = Some(data.to_vec());
            }
        });

        to_send.take().map(|data| {
            conn.send_bytes(&data).unwrap();
        });

        conn.check_for_timeouts(|msg_id| println!("{:?} timed out", msg_id));
    }
}
