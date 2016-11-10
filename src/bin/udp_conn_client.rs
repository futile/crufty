extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{UdpSocketImpl, UdpConnection, UdpConnectionEvent};

fn main() {
    use crufty::net::udp::MockUdpSocket;

    let mut socket = MockUdpSocket::bind(&"127.0.0.1:12365".parse().unwrap()).unwrap();

    socket.latency = Duration::from_millis(250);
    socket.jitter = Duration::from_millis(0);
    socket.packet_loss_ratio = 0.;

    let mut conn = UdpConnection::from_socket(socket,
                                              &"127.0.0.1:12366".parse().unwrap(),
                                              Duration::from_secs(1));

    // interval at which we send messages
    let send_interval = Duration::from_millis(60);

    let mut event_buffer = Vec::new();

    loop {
        let msg: &[u8] = &[b'x'; 100];
        let sent_id = conn.send_bytes(&msg);

        println!("sent request, id: {:?}", sent_id);

        conn.update(Instant::now() + send_interval, &mut event_buffer);

        for event in event_buffer.drain(..) {
            match event {
                UdpConnectionEvent::MessageTimedOut(msg_id) => println!("timed out: {:?}", msg_id),
                UdpConnectionEvent::MessageReceived { data: _, new_acks } => {
                    println!("new acks: {:?}", new_acks);
                }
            }
        }
    }
}
