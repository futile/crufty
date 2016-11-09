use std::time::{Instant, Duration};
use std::cell::RefCell;
use std::cmp::{Ord, PartialOrd, Eq, PartialEq, Ordering};
use std::collections::BinaryHeap;
use std::net::{SocketAddr, UdpSocket};
use std::io;

use rand::{self, Rng};

#[derive(Debug)]
struct Buffer(Vec<u8>);

impl Buffer {
    fn new() -> Buffer {
        Buffer(Vec::new())
    }

    fn take(&mut self) -> Vec<u8> {
        ::std::mem::replace(&mut self.0, Vec::new())
    }

    fn done(&mut self, buf: Vec<u8>) {
        if buf.capacity() > self.0.capacity() {
            self.0 = buf;

            // invariant: buffer has to stay cleared
            self.0.clear();
        }
    }
}

struct PacketInfo {
    original_recv_time: Instant,
    mocked_recv_time: Instant,
    data: Vec<u8>,
}

impl PartialEq for PacketInfo {
    fn eq(&self, other: &Self) -> bool {
        self.mocked_recv_time.eq(&other.mocked_recv_time)
    }
}

impl Eq for PacketInfo {}

impl PartialOrd for PacketInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cmp(self))
    }
}

impl Ord for PacketInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        // inverse ordering for our BinaryHeap
        self.mocked_recv_time.cmp(&other.mocked_recv_time).reverse()
    }
}

pub struct MockUdpSocket {
    socket: UdpSocket,

    // mocked properties of the connection
    pub latency: Duration,
    pub jitter: Duration,
    pub packet_loss_ratio: f32,

    buffer: RefCell<Buffer>,
    timeout: RefCell<Option<Duration>>,
    received_packets: RefCell<BinaryHeap<PacketInfo>>,
}

impl MockUdpSocket {
    pub fn bind(local: &SocketAddr) -> io::Result<MockUdpSocket> {
        let inner = UdpSocket::bind(local)?;

        Ok(MockUdpSocket {
            socket: inner,

            latency: Duration::from_millis(0),
            jitter: Duration::from_millis(0),
            packet_loss_ratio: 0f32,

            buffer: RefCell::new(Buffer::new()),
            timeout: RefCell::new(None),
            received_packets: RefCell::new(BinaryHeap::new()),
        })
    }

    pub fn connect(&self, remote: &SocketAddr) -> io::Result<()> {
        self.socket.connect(remote)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.socket.send(buf)
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        *self.timeout.borrow_mut() = dur;

        // always succeeds
        Ok(())
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut packets = self.received_packets.borrow_mut();
        let mut now = Instant::now();
        let timeout = self.timeout.borrow().clone();
        let start = now;

        while timeout.map_or(true, |to| now < start + to) {
            // check received_packets for packets that are now ready to be returned,
            // or within timeout_dur
            if packets.peek().map_or(false, |info| {
                timeout.map_or(true, |to| now + to >= info.mocked_recv_time)
            }) {
                // extract first packet, which is guarded by this if
                let info = packets.pop().unwrap();

                // check if we receive the packet within the timeout, but not yet
                if info.mocked_recv_time > now {
                    // if so, sleep until received
                    ::std::thread::sleep(info.mocked_recv_time - now);
                }

                // check if the buffer is big enough
                if buf.len() < info.data.len() {
                    panic!("MockUdpSocket::recv(): buf.len() < info.data.len()");
                }

                // copy packet to user buffer
                buf[..info.data.len()].copy_from_slice(&info.data[..]);

                // return how much data is available
                return Ok(info.data.len());
            }

            // temporary timeout, to stay within our deadline
            let temp_timeout = timeout.map(|to| (start + to) - now);
            self.socket.set_read_timeout(temp_timeout)?;

            // get a buffer for receiving
            let mut buffer = self.buffer.borrow_mut().take();

            // see conn.rs for why we do unsafe here
            unsafe {
                let max_udp_size: usize = 65507;
                buffer.reserve(max_udp_size);
                buffer.set_len(max_udp_size);

                // try to actually receive a packet
                match self.socket.recv(&mut buffer) {
                    // got one!
                    Ok(bytes_read) => {
                        assert!(bytes_read <= buffer.capacity());
                        buffer.set_len(bytes_read);

                        let mut rng = rand::thread_rng();

                        // check if we should receive or drop this packet
                        // simulates packet loss
                        if rng.gen_range(0.0, 1.0) >= self.packet_loss_ratio {
                            // yes, receive

                            // shrink, for better mem usage
                            buffer.shrink_to_fit();

                            // calculate a mock receive time based on latency and jitter
                            let mock_recv_time =
                                Instant::now() + self.latency +
                                Duration::new(0, rng.gen_range(0, self.jitter.subsec_nanos()));

                            // add to heap
                            packets.push(PacketInfo {
                                original_recv_time: now,
                                mocked_recv_time: mock_recv_time,
                                data: buffer,
                            });
                        } else {
                            // no, drop packet
                            println!("packet dropped");

                            // return buffer
                            self.buffer.borrow_mut().done(buffer);
                        }

                    }
                    // error, either timeout or other
                    e @ Err(_) => {
                        buffer.set_len(0);

                        // return buffer
                        self.buffer.borrow_mut().done(buffer);

                        // forward error to caller, we are done
                        return e;
                    }
                }
            }

            // update current time, because recv() might have taken time
            now = Instant::now();
        }

        Err(io::Error::new(io::ErrorKind::WouldBlock, "mock read timed out"))
    }
}
