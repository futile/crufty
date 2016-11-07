use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::io::{self, Cursor};
use std::io::prelude::{Write, Read};
use std::collections::VecDeque;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use super::seqnum::SequenceNumber;
use super::ackstat::AckStatus;

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

pub struct UdpConnection {
    socket: UdpSocket,
    remote_addr: SocketAddr,
    next_local_sequence_number: SequenceNumber,
    ack_control: AckStatus ,
    rtt: Duration,

    next_message_id: MessageId,
    buffer: Buffer,
    pending_acks: VecDeque<InFlightInfo>,
}

impl UdpConnection {
    pub fn new(local_addr: &SocketAddr, remote_addr: SocketAddr) -> UdpConnection {
        let socket = UdpSocket::bind(local_addr).unwrap();

        UdpConnection {
            socket: socket,
            remote_addr: remote_addr,
            next_local_sequence_number: SequenceNumber::first(),
            ack_control: AckStatus ::new(),
            rtt: Duration::new(0, 0),

            next_message_id: MessageId(0),
            buffer: Buffer::new(),
            pending_acks: VecDeque::new(),
        }
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
        let magic = reader.read_u32::<Encoding>()?;

        if magic != MAGIC_PROTOCOL_ID {
            return Err(io::Error::new(io::ErrorKind::Other, "wrong protocol id"));
        }

        let seq_num = SequenceNumber::read_from_packet::<Encoding, _>(reader)?;
        let acks = AckStatus::read_from_packet::<Encoding, _>(reader)?;

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
        let sent_count = self.socket.send_to(&buffer, &self.remote_addr).unwrap();
        assert_eq!(sent_count, buffer.len(), "only a partial send occured, should not happen??");

        // create MessageId
        let msg_id = self.next_message_id;
        self.next_message_id.0 += 1;

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
}
