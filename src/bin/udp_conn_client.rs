extern crate crufty;

use std::time::{Duration};

use crufty::net::udp::{self, UdpSocketImpl, UdpConnection, ReceiveEvent};

fn main() {
    use crufty::net::udp::MockUdpSocket;

    let mut socket = MockUdpSocket::bind(&"127.0.0.1:12365".parse().unwrap()).unwrap();

    socket.latency = Duration::from_millis(0);
    socket.jitter = Duration::from_millis(0);
    socket.packet_loss_ratio = 0.;

    let mut conn = UdpConnection::from_socket(socket,
                                              &"127.0.0.1:12366".parse().unwrap(),
                                              Duration::from_secs(3));

    // interval at which we send messages
    let send_interval = Duration::from_millis(100);

    let msg: &[u8] = &[b'x'; 100];

    loop {
        conn.send_bytes(&msg).unwrap();

        // let mut now = Instant::now();
        // let deadline = now + send_interval;
        // while now < deadline {
        //     conn.recv_with_timeout(Some(deadline - now), |e| match e {
        //         ReceiveEvent::NewAck(msg_id, rtt) => {
        //             println!("{:?} done, took {}ms", msg_id, udp::dur_to_ms(&rtt));
        //         }
        //         ReceiveEvent::NewData(_) => {}
        //     });

        //     now = Instant::now();
        // }

        conn.recv_with_timeout(Some(send_interval), |e| match e {
            ReceiveEvent::NewAck(msg_id, rtt) => {
                println!("{:?} done, took {}ms", msg_id, udp::dur_to_ms(&rtt));
            }
            ReceiveEvent::NewData(_) => {}
        });

        conn.check_for_timeouts(|msg_id| println!("{:?} timed out", msg_id));
    }
}
