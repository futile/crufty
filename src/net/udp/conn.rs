use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::io::{self, Cursor};
use std::io::prelude::{Write, Read};
use std::collections::VecDeque;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::seqnum::SequenceNumber;
use super::ackstat::AckStatus;

// for mocking
use super::UdpSocketImpl;

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

#[derive(Debug, Clone)]
struct InFlightInfo {
    seq_num: SequenceNumber,
    sent_time: Instant,
    msg_id: MessageId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MessageId(u64);

const MAGIC_PROTOCOL_ID: u32 = 0xABFECDFE;

type Encoding = BigEndian;

#[derive(Debug, Copy, Clone)]
struct PacketHeader {
    seq_num: SequenceNumber,
    acks: AckStatus ,
}

#[derive(Debug)]
pub enum UdpConnectionEvent {
    MessageReceived(Vec<u8>),
    MessageTimedOut(MessageId),
}

pub struct UdpConnection<S: UdpSocketImpl> {
    socket: S,
    next_local_sequence_number: SequenceNumber,
    ack_control: AckStatus ,
    averaged_rtt: Duration,

    next_message_id: MessageId,
    buffer: Buffer,
    pending_acks: VecDeque<InFlightInfo>,
    packet_timeout_limit: Duration,
}

impl<S: UdpSocketImpl> UdpConnection<S> {
    pub fn new(local_addr: &SocketAddr, remote_addr: &SocketAddr, packet_timeout_limit: Duration) -> UdpConnection<S> {
        let socket = S::bind(local_addr).unwrap();

        UdpConnection::from_socket(socket, remote_addr, packet_timeout_limit)
    }

    pub fn from_socket(socket: S, remote_addr: &SocketAddr, packet_timeout_limit: Duration) -> UdpConnection<S> {
        socket.connect(remote_addr).unwrap();

        UdpConnection {
            socket: socket,
            next_local_sequence_number: SequenceNumber::first(),
            ack_control: AckStatus ::new(),
            averaged_rtt: Duration::new(0, 0),

            next_message_id: MessageId(0),
            buffer: Buffer::new(),
            pending_acks: VecDeque::new(),
            packet_timeout_limit: packet_timeout_limit,
        }
    }

    /// Changes the duration after which an un-acked packet is considered lost.
    pub fn set_packet_timeout_limit(&mut self, new_timeout: Duration) {
        self.packet_timeout_limit = new_timeout;
    }

    fn write_header<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        // write magic protocol id
        writer.write_u32::<Encoding>(MAGIC_PROTOCOL_ID)?;

        // write sequence number
        self.next_local_sequence_number.write_to_packet::<Encoding, _>(writer)?;

        // write acks
        self.ack_control.write_to_packet::<Encoding, _>(writer)
    }

    fn read_header<R: Read>(reader: &mut R) -> io::Result<PacketHeader> {
        // first is the magic protocol id
        let magic = reader.read_u32::<Encoding>()?;

        // if it doesn't match, return an error
        if magic != MAGIC_PROTOCOL_ID {
            return Err(io::Error::new(io::ErrorKind::Other, "wrong protocol id"));
        }

        // deserialize sequence number and acks
        let seq_num = SequenceNumber::read_from_packet::<Encoding, _>(reader)?;
        let acks = AckStatus::read_from_packet::<Encoding, _>(reader)?;

        // build header and return
        Ok(PacketHeader {
            seq_num: seq_num,
            acks: acks,
        })
    }

    pub fn send_bytes(&mut self, msg: &[u8]) -> MessageId {
        let mut buffer = self.buffer.take();

        // write header to buffer
        self.write_header(&mut buffer).unwrap();

        // write msg to buffer
        buffer.write_all(msg).unwrap();

        // send packet
        let sent_count = self.socket.send(&buffer).unwrap();

        // sanity check, should return an error if we try to send too much (which the unwrap above should catch)
        assert_eq!(sent_count, buffer.len(), "only a partial send occured, should not happen??");

        // create and increase MessageId
        let msg_id = self.next_message_id;
        self.next_message_id.0 += 1;

        println!("sending: {:?}", msg_id);

        // add packet info (seq num, sent-timestamp, message id) to pending acks
        self.pending_acks.push_back(InFlightInfo {
            seq_num: self.next_local_sequence_number,
            sent_time: Instant::now(),
            msg_id: msg_id,
        });

        // increase next sequence number
        self.next_local_sequence_number += 1.into();

        // give buffer back
        self.buffer.done(buffer);

        return msg_id;
    }

    // TODO this should probably return a Result, so we know when e.g. a wrong magic number arrived
    fn receive_packet(&mut self, buffer: &[u8]) -> Option<UdpConnectionEvent> {
        let mut reader = Cursor::new(buffer);

        // read header
        let header = match Self::read_header(&mut reader) {
            Ok(header) => header,
            Err(e) => {
                println!("warning: UdpConnection::read_header() returned '{}', dropping packet", e);
                return None;
            }
        };

        // ack the remote packet
        self.ack_control.ack(header.seq_num);

        // have to take this out of the closure
        let mut new_average_rtt = self.averaged_rtt;

        // remove acked packages from pending acks and update average rtt
        self.pending_acks.retain(|info| {
            if header.acks.is_acked(info.seq_num) {
                // update average rtt by 10% towards this packet's rtt (see link at module top)
                new_average_rtt = (new_average_rtt * 9 + info.sent_time.elapsed()) / 10;

                println!("received {:?}", info.msg_id);

                // packet was acked, don't keep it in pending queue
                return false;
            }

            // packet not acked yet, keep as pending
            true
        });

        // update average rtt
        self.averaged_rtt = new_average_rtt;

        // save until where we've read the buffer
        let reader_pos = reader.position() as usize;

        // create and return event which contains the rest of the message
        Some(UdpConnectionEvent::MessageReceived(reader.into_inner()[reader_pos..].to_vec()))
    }

    fn recv_with_timeout(&mut self, timeout: Duration) -> Option<UdpConnectionEvent> {
        // sanity-check warning
        if timeout == Duration::new(0, 0) {
            println!("warning: UdpConnection::update(): 'timeout' is zero -> blocking read");
        }

        // set timeout for the receive
        self.socket.set_read_timeout(Some(timeout)).unwrap();

        // get a buffer for receiving
        let mut buffer = self.buffer.take();

        // resize the buffer without allocating, because we want >64kb
        unsafe {
            // see e.g. https://stackoverflow.com/questions/1098897/what-is-the-largest-safe-udp-packet-size-on-the-internet
            let max_udp_size: usize = 65507;

            // reserve, i.e. allocate, enough memory
            // this is necessary so socket.recv() won't throw away anything..
            buffer.reserve(max_udp_size);

            // this is unsafe
            buffer.set_len(max_udp_size);

            // try receiving a packet (with the timeout set before)
            let maybe_event = match self.socket.recv(&mut buffer) {
                // receive successful
                Ok(bytes_read) => {
                    // sanity check
                    assert!(bytes_read <= buffer.capacity());

                    // **IMPORTANT** restrict buffer size to how much is actually valid
                    buffer.set_len(bytes_read);

                    // this is safe now, since we made sure the buffer size is correct
                    self.receive_packet(&buffer)
                },
                Err(e) => {
                    // **IMPORTANT** on any error, set buffer length to 0
                    buffer.set_len(0);

                    // see what error we got
                    match e.kind() {
                        // on timeout, just return no packet
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => None,
                        // else, panic
                        _ => panic!(e),
                    }
                }
            };

            // return buffer (this will also clear() it)
            self.buffer.done(buffer);

            // return (maybe) an event
            maybe_event
        }
    }

    fn check_for_timeouts(&mut self, event_buffer: &mut Vec<UdpConnectionEvent>) {
        let now = Instant::now();

        // borrow checker forces this copy, or I couldn't do it better
        let timeout_limit = self.packet_timeout_limit;

        // retain only not-timeouted packets
        self.pending_acks.retain(|info| {
            // packet is timed out if it has been in flight for more than a certain time
            let timed_out = now.duration_since(info.sent_time) >= timeout_limit;

            // if it timed out, report an event to the caller
            if timed_out {
                event_buffer.push(UdpConnectionEvent::MessageTimedOut(info.msg_id));
            }

            // this cleans out packets that have timed out
            !timed_out
        });
    }

    pub fn update(&mut self, deadline: Instant, event_buffer: &mut Vec<UdpConnectionEvent>) {
        // reserve some time for bookkeeping
        let deadline = deadline - Duration::new(0, 100000); // 0.1ms

        // try to receive packets until we are at our deadline
        loop {
            let now = Instant::now();

            // are we at our deadline already?
            if now >= deadline {
                // if so, then don't try receiving again
                break;
            }

            // calculate timeout until deadline
            let timeout = deadline - now;

            // see if there is a new message
            if let Some(event) = self.recv_with_timeout(timeout) {
                // save message for caller
                event_buffer.push(event);
            }
        }

        // done with receiving, check for timeouted packages now
        self.check_for_timeouts(event_buffer);
    }
}
