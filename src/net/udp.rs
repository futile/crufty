use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::num::Wrapping;
use std::ops::Sub;

// this is an implementation following http://gafferongames.com/networking-for-game-programmers/reliability-and-flow-control/

type SequenceNumberPrecision = u16;

custom_derive! {
    #[derive(NewtypeAdd, NewtypeAdd(&self), NewtypeAddAssign, PartialOrd, Ord, PartialEq, Eq, Debug, Clone, Copy)]
    struct SequenceNumber(Wrapping<SequenceNumberPrecision>);
}

impl SequenceNumber {
    const FIRST_SEQUENCE_NUMBER: SequenceNumberPrecision = 1;

    pub fn new(seq_num: SequenceNumberPrecision) -> SequenceNumber {
        SequenceNumber(Wrapping(seq_num))
    }

    pub fn first() -> SequenceNumber {
        SequenceNumber::new(Self::FIRST_SEQUENCE_NUMBER)
    }
}

impl<'a> Sub<&'a SequenceNumber> for SequenceNumber {
    type Output = i32;

    fn sub(self, rhs: &SequenceNumber) -> i32 {
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

impl Sub for SequenceNumber {
    type Output = i32;

    fn sub(self, rhs: SequenceNumber) -> i32 {
        self - &rhs
    }
}

impl From<SequenceNumberPrecision> for SequenceNumber {
    fn from(val: SequenceNumberPrecision) -> SequenceNumber {
        SequenceNumber::new(val)
    }
}

type EarlierAcksBitfield = u32;

struct AckControl {
    remote_sequence_number: Option<SequenceNumber>,
    earlier_acks: EarlierAcksBitfield,
}

impl AckControl {
    pub fn new() -> AckControl {
        AckControl {
            remote_sequence_number: None,
            earlier_acks: 0,
        }
    }

    pub fn ack(&mut self, seq_num: SequenceNumber) {
        let remote_sequence_number = match self.remote_sequence_number {
            Some(ref mut rsn) => rsn,
            None => { self.remote_sequence_number = Some(seq_num); return; }
        };

        // this substraction is wraparound aware
        let diff: i32 = seq_num - (remote_sequence_number as &SequenceNumber);

        println!("diff: {}", diff);

        if diff > 0 {
            // a newer, more recent remote sequence number was received

            // add current `remote_sequence_number` to `earlier_acks`, as lowest bit
            // only do this if this wasn't the first ack. TODO remove this check somehow
            self.earlier_acks <<= 1;
            self.earlier_acks |= 0x1;

            // mark all other sequence numbers in between as un-acked
            let (new_acks, overflowed) = self.earlier_acks.overflowing_shl(diff as u32 - 1);
            self.earlier_acks = new_acks;

            // check if we have skipped too many sequence numbers for our bitfield to save
            if overflowed {
                println!("warning: AckControl::ack(): skipped sequence numbers due to too big diff: {}", diff);
            }

            // set `seq_num` as the most recent remote sequence number we have acked
            *remote_sequence_number = seq_num;
        } else if diff < 0 {
            // a packet with an older sequence number was received

            // check if this packet is too old to be acked by us
            if diff < -(::std::mem::size_of::<EarlierAcksBitfield>() as i32 * 8) {
                println!("warning: AckControl::ack(): can't save ack for an old packet, diff: {}", diff);
                return;
            }

            // at which offset do we have to store this ack in our bitmask?
            let offset = diff.abs() - 1;

            // sanity check: make sure we didn't receive a packet with the same sequence number before
            if (self.earlier_acks & 0x1 << offset) > 0 {
                // a sequence number was received at least twice
                println!("warning: AckControl::ack(): duplicate (old) sequence number received, ignoring.");
                return;
            }

            // mark this sequence number as acked
            self.earlier_acks |= 0x1 << offset;
        } else {
            // a sequence number was received at least twice
            println!("warning: AckControl::ack(): duplicate sequence number received, ignoring.");
            return;
        }
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
    use super::{SequenceNumber, AckControl};

    #[test]
    fn test_sub_seq_nums() {
        // regular cases
        assert_eq!(SequenceNumber::new(10) - SequenceNumber::new(5), 5);
        assert_eq!(SequenceNumber::new(5) - SequenceNumber::new(10), -5);

        // wrapping cases
        assert_eq!(SequenceNumber::new(5) - SequenceNumber::new(65535), 6);
        assert_eq!(SequenceNumber::new(65535) - SequenceNumber::new(5), -6);
    }

    #[test]
    fn test_ack_control_basic() {
        let mut acks = AckControl::new();

        // empty in the beginning
        assert_eq!(acks.remote_sequence_number, None);
        assert_eq!(acks.earlier_acks, 0);

        let mut seq_num = SequenceNumber::first();

        // ack a packet
        acks.ack(seq_num);
        assert_eq!(acks.remote_sequence_number.unwrap(), seq_num);
        assert_eq!(acks.earlier_acks, 0);

        // ack another
        seq_num += 1.into();
        acks.ack(seq_num);
        assert_eq!(acks.remote_sequence_number.unwrap(), seq_num);
        assert_eq!(acks.earlier_acks, 1);

        // ack with a gap
        seq_num += 2.into();
        acks.ack(seq_num);
        assert_eq!(acks.remote_sequence_number.unwrap(), seq_num);
        assert_eq!(acks.earlier_acks, 0b110);
    }

    #[test]
    fn test_ack_control_wraparound() {
        let mut acks = AckControl::new();

        let mut seq_num = 65535.into();

        acks.ack(seq_num);

        seq_num += 10.into();
        acks.ack(seq_num);
        assert_eq!(acks.remote_sequence_number.unwrap(), seq_num);
        assert_eq!(acks.earlier_acks, 1 << 9);
    }
}
