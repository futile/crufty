// this is an implementation following http://gafferongames.com/networking-for-game-programmers/reliability-and-flow-control/

mod seqnum;
mod ackstat;
mod conn;
mod mock;

pub use self::conn::{MessageId, UdpConnection, ReceiveEvent};
pub use self::mock::MockUdpSocket;

use std::net::{SocketAddr};
use std::io;
use std::time::Duration;


pub fn ns_to_ms(ns: u32) -> u64 {
    ns as u64 / 1_000_000
}

pub fn dur_to_ms(dur: &Duration) -> u64 {
    dur.as_secs() * 1000 + ns_to_ms(dur.subsec_nanos())
}

// trait to unify ::std::net::UdpSocket and our MockUdpSocket
pub trait UdpSocketImpl: Sized {
    fn bind(local: &SocketAddr) -> io::Result<Self>;
    fn connect(&self, remote: &SocketAddr) -> io::Result<()>;
    fn send(&self, buf: &[u8]) -> io::Result<usize>;
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>;
    fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;
}

impl UdpSocketImpl for ::std::net::UdpSocket {
    fn bind(local: &SocketAddr) -> io::Result<Self> {
        Self::bind(local)
    }

    fn connect(&self, remote: &SocketAddr) -> io::Result<()>{
        self.connect(remote)
    }

    fn send(&self, buf: &[u8]) -> io::Result<usize>{
        self.send(buf)
    }

    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>{
        self.set_read_timeout(dur)
    }

    fn recv(&self, buf: &mut [u8]) -> io::Result<usize>{
        self.recv(buf)
    }
}
