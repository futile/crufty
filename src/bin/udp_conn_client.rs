extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{self, CongestionControl, UdpConnection, ReceiveEvent};

fn main() {
    let mut conn = CongestionControl::new(UdpConnection::new(&"127.0.0.1:12365".parse().unwrap(),
                                                             &"127.0.0.1:12366".parse().unwrap(),
                                                             Duration::from_secs(3)));

    // interval at which we send messages
    let send_interval = Duration::from_millis(100);

    let msg: &[u8] = &[b'x'; 100];

    loop {
        conn.send_bytes(&msg).unwrap();

        let mut now = Instant::now();
        let deadline = now + send_interval;
        while now < deadline {
            conn.recv_with_timeout(Some(deadline - now), |e| match e {
                ReceiveEvent::NewAck(msg_id, rtt) => {
                    // println!("{:?} done, took {}ms", msg_id, udp::dur_to_ms(&rtt));
                }
                ReceiveEvent::NewData(_) => {}
            });

            now = Instant::now();
        }

        conn.check_for_timeouts(|msg_id| println!("{:?} timed out", msg_id));
    }
}
