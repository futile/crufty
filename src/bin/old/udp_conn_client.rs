extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{self, CongestionControl, CongestionStatus, ReliabilityWrapper, ReceiveEvent};

fn main() {
    let mut conn = CongestionControl::new(ReliabilityWrapper::new(&"127.0.0.1:12365".parse().unwrap(),
                                                             &"127.0.0.1:12366".parse().unwrap(),
                                                             Duration::from_secs(3)));

    let msg: &[u8] = &[b'x'; 100];

    loop {
        let cong_status = conn.congestion_status();

        if let CongestionStatus::ReadyToSend = cong_status {
            conn.send_bytes(&msg).unwrap();
        } else {
            let timeout = cong_status.wait_time().unwrap();

            conn.recv_with_timeout(Some(timeout), |e| {
                match e {
                    ReceiveEvent::NewAck(_msg_id, _rtt) => {
                        // println!("{:?} done, took {}ms", msg_id, udp::dur_to_ms(&rtt));
                    }
                    ReceiveEvent::NewData(_) => {}
                }
            });
        }

        conn.check_for_timeouts(|msg_id| println!("{:?} timed out", msg_id));
    }
}
