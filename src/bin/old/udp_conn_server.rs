extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{self, CongestionControl, ReliabilityWrapper, ReceiveEvent};

fn main() {
    let mut conn = CongestionControl::new(ReliabilityWrapper::new(&"127.0.0.1:12366".parse().unwrap(),
                                                             &"127.0.0.1:12365".parse().unwrap(),
                                                             Duration::from_secs(3)));
    let mut last_received = Instant::now();
    let mut to_send = None;
    let to_send = &mut to_send;
    loop {
        conn.recv_with_timeout(None, |e| match e {
            ReceiveEvent::NewAck(_msg_id, _rtt) => {
                // println!("{:?} done, took {}ms", msg_id, udp::dur_to_ms(&rtt));
            }
            ReceiveEvent::NewData(data) => {
                println!("received something, after {}ms", udp::dur_to_ms(&last_received.elapsed()));
                last_received = Instant::now();
                *to_send = Some(data.to_vec());
            }
        });

        to_send.take().map(|data| {
            conn.send_bytes(&data).unwrap();
        });

        conn.check_for_timeouts(|msg_id| println!("{:?} timed out", msg_id));
    }
}
