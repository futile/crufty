use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::num::Wrapping;
use std::ops::Sub;

// this is an implementation following http://gafferongames.com/networking-for-game-programmers/reliability-and-flow-control/

type SequenceNumberPrecision = u16;

custom_derive! {
    #[derive(NewtypeAdd, PartialOrd, Ord, PartialEq, Eq, Debug, Clone, Copy)]
    struct SequenceNumber(Wrapping<SequenceNumberPrecision>);
}

impl SequenceNumber {
    const FIRST_SEQUENCE_NUMBER: SequenceNumberPrecision = 1;

    pub fn new(seq_num: u16) -> SequenceNumber {
        SequenceNumber(Wrapping(seq_num))
    }

    pub fn first() -> SequenceNumber {
        SequenceNumber::new(Self::FIRST_SEQUENCE_NUMBER)
    }
}

impl Sub for SequenceNumber {
    type Output = i32;

    fn sub(self, rhs: SequenceNumber) -> i32 {
        const SEQ_HALF_DIFF: i32 = SequenceNumberPrecision::max_value() as i32 / 2;

        let diff = (self.0).0 as i32 - (rhs.0).0 as i32;

        if diff >= 0 {
            if diff <= SEQ_HALF_DIFF {
                diff
            } else {
                -1 * ((rhs.0 - self.0).0 as i32)
            }
        } else {
            if diff.abs() <= SEQ_HALF_DIFF {
                diff
            } else {
                (self.0 - rhs.0).0 as i32
            }
        }
    }
}

struct AckControl {
    remote_sequence_number: SequenceNumber,
    earlier_acks: u16,
}

impl AckControl {
    pub fn new() -> AckControl {
        AckControl {
            remote_sequence_number: SequenceNumber::first(),
            earlier_acks: 0xffff,
        }
    }

    pub fn ack(&mut self, seq_num: SequenceNumber) {
        let diff = seq_num - self.remote_sequence_number;
    }
}

pub struct UdpConnection {
    socket: UdpSocket,
    remote_addr: SocketAddr,
    next_local_sequence_number: SequenceNumber,
    ack_control: AckControl,
    rtt: Duration,
}

impl UdpConnection {
    pub fn new(local_addr: &SocketAddr, remote_addr: SocketAddr) -> UdpConnection {
        let socket = UdpSocket::bind(local_addr).unwrap();

        UdpConnection {
            socket: socket,
            remote_addr: remote_addr,
            next_local_sequence_number: SequenceNumber::first(),
            ack_control: AckControl::new(),
            rtt: Duration::new(0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SequenceNumber;

    #[test]
    fn test_sub_seq_nums() {
        // regular cases
        assert_eq!(SequenceNumber::new(10) - SequenceNumber::new(5), 5);
        assert_eq!(SequenceNumber::new(5) - SequenceNumber::new(10), -5);

        // wrapping cases
        assert_eq!(SequenceNumber::new(5) - SequenceNumber::new(65535), 6);
        assert_eq!(SequenceNumber::new(65535) - SequenceNumber::new(5), -6);
    }
}
