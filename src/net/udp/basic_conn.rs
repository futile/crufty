use std::time::Duration;
use std::net::{UdpSocket, SocketAddr};
use std::io;

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

pub struct BasicUdpConnection {
    pub socket: UdpSocket,
    buffer: Buffer,
}

impl BasicUdpConnection {
    pub fn new(socket: UdpSocket) -> BasicUdpConnection {
        BasicUdpConnection {
            socket: socket,
            buffer: Buffer::new(),
        }
    }

    pub fn recv_with_timeout(&mut self,
                             timeout: Option<Duration>)
                             -> Option<io::Result<(Vec<u8>, SocketAddr)>> {
        // sanity-check warning
        timeout.map(|to| {
            assert_ne!(to,
                       Duration::new(0, 0),
                       "zero-duration is invalid, use 'None' for blocking")
        });

        // set timeout for the receive
        self.socket.set_read_timeout(timeout).unwrap();

        // see e.g. https://stackoverflow.com/questions/1098897/
        let max_udp_size: usize = 65507;

        // get a buffer for receiving
        let mut buffer = self.buffer.take();

        // reserve, i.e. allocate, enough memory
        // this is necessary so socket.recv() won't throw away anything..
        buffer.reserve(max_udp_size);

        // resize the buffer without allocating, because we want >64kb
        unsafe {
            // this is unsafe
            buffer.set_len(max_udp_size);

            // try receiving a packet (with the timeout set before)
            match self.socket.recv_from(&mut buffer) {
                // receive successful
                Ok((bytes_read, addr)) => {
                    // sanity check
                    assert!(bytes_read <= buffer.capacity());

                    // **IMPORTANT** restrict buffer size to how much is actually valid
                    buffer.set_len(bytes_read);

                    // this is safe now, since we made sure the buffer size is correct
                    Some(Ok((buffer, addr)))
                }
                Err(e) => {
                    // **IMPORTANT** on any error, set buffer length to 0
                    buffer.set_len(0);

                    // return buffer (this will also clear() it)
                    // this prevents timeout/error cases from consuming it,
                    // so we can reuse it for the next receive (especially for timeouts)
                    self.buffer.done(buffer);

                    // see what error we got
                    match e.kind() {
                        // on timeout do nothing
                        io::ErrorKind::WouldBlock |
                        io::ErrorKind::TimedOut => None,
                        // else, return the error
                        _ => Some(Err(e)),
                    }
                }
            }
        }
    }

    pub fn send(&self, payload: &[u8]) -> io::Result<()> {
        // send packet
        let sent_count = self.socket.send(&payload)?;

        // sanity check, should return an error if we try to send too much
        // (which the unwrap above should catch)
        assert_eq!(sent_count,
                   payload.len(),
                   "only a partial send occured, should not happen??");

        Ok(())
    }
}
