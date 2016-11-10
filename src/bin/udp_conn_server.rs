extern crate crufty;

use std::time::{Instant, Duration};

use crufty::net::udp::{UdpConnection, UdpConnectionEvent, UdpSocketImpl};

fn main() {
    use crufty::net::udp::MockUdpSocket;

    let mut socket = MockUdpSocket::bind(&"127.0.0.1:12366".parse().unwrap()).unwrap();

    socket.latency = Duration::from_millis(250);
    socket.jitter = Duration::from_millis(0);
    socket.packet_loss_ratio = 0.;

    let mut conn = UdpConnection::from_socket(socket,
                                              &"127.0.0.1:12365".parse().unwrap(),
                                              Duration::from_secs(1));

    let mut event_buffer = vec![];

    loop {
        conn.update(Instant::now() + Duration::from_millis(20), &mut event_buffer);

        for event in event_buffer.drain(..) {
            match event {
                UdpConnectionEvent::MessageReceived{ data, new_acks } => {
                    // let data_str = ::std::str::from_utf8(&data).unwrap();

                    let sent_id = conn.send_bytes(&data);

                    println!("sent resp., id: {:?}", sent_id);
                    println!("new acks: {:?}", new_acks);
                }
                mto @ UdpConnectionEvent::MessageTimedOut(_) => println!("{:?}", mto),
            }
        }
    }
}
