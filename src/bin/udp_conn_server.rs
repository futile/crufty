extern crate crufty;

use std::time::{Duration};

use crufty::net::udp::{self, UdpConnection, UdpSocketImpl, ReceiveEvent};

fn main() {
    use crufty::net::udp::MockUdpSocket;

    let mut socket = MockUdpSocket::bind(&"127.0.0.1:12366".parse().unwrap()).unwrap();

    socket.latency = Duration::from_millis(0);
    socket.jitter = Duration::from_millis(0);
    socket.packet_loss_ratio = 0.0;

    let mut conn = UdpConnection::from_socket(socket,
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
