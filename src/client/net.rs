use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use std::io;

use net::udp::{BasicUdpConnection, ReliabilityWrapper, ConnectionWrapper};

pub struct NetworkInterface {
    pub conn: BasicUdpConnection,
    pub wrapper: ConnectionWrapper,
}

impl NetworkInterface {
    pub fn new(local_addr: &SocketAddr, remote_addr: &SocketAddr) -> io::Result<NetworkInterface> {
        let socket = UdpSocket::bind(local_addr)?;
        socket.connect(remote_addr)?;

        let wrapper = ConnectionWrapper {
            cong_control: None,
            reliability: ReliabilityWrapper::new(local_addr, remote_addr, Duration::from_secs(1)),
        };

        Ok(NetworkInterface {
            conn: BasicUdpConnection::new(socket),
            wrapper: wrapper,
        })
    }
}
