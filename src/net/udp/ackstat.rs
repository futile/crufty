use std::io;
use std::io::prelude::{Write, Read};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

use super::seqnum::SequenceNumber;

type EarlierAcksBitfield = u32;

#[derive(Debug, Copy, Clone)]
pub struct AckStatus {
    remote_sequence_number: Option<SequenceNumber>,
    earlier_acks: EarlierAcksBitfield,
}

impl AckStatus  {
    pub fn new() -> AckStatus  {
        AckStatus  {
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
                println!("warning: AckStatus ::ack(): skipped sequence numbers due to too big diff: {}", diff);
            }

            // set `seq_num` as the most recent remote sequence number we have acked
            *remote_sequence_number = seq_num;
        } else if diff < 0 {
            // a packet with an older sequence number was received

            // check if this packet is too old to be acked by us
            if diff < -(::std::mem::size_of::<EarlierAcksBitfield>() as i32 * 8) {
                println!("warning: AckStatus ::ack(): can't save ack for an old packet, diff: {}", diff);
                return;
            }

            // at which offset do we have to store this ack in our bitmask?
            let offset = diff.abs() - 1;

            // sanity check: make sure we didn't receive a packet with the same sequence number before
            if (self.earlier_acks & 0x1 << offset) > 0 {
                // a sequence number was received at least twice
                println!("warning: AckStatus ::ack(): duplicate (old) sequence number received, ignoring.");
                return;
            }

            // mark this sequence number as acked
            self.earlier_acks |= 0x1 << offset;
        } else {
            // a sequence number was received at least twice
            println!("warning: AckStatus ::ack(): duplicate sequence number received, ignoring.");
            return;
        }
    }

    pub fn write_to_packet<O: ByteOrder, W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let rsn = match self.remote_sequence_number {
            Some(rsn) => rsn,
            None => 0.into(),
        };

        rsn.write_to_packet::<O, _>(writer)?;
        writer.write_u32::<O>(self.earlier_acks)
    }

    pub fn read_from_packet<O: ByteOrder, R: Read>(reader: &mut R) -> io::Result<AckStatus > {
        let rsn = SequenceNumber::read_from_packet::<O, _>(reader)?;
        let ack_bits = reader.read_u32::<O>()?;

        Ok(AckStatus {
            remote_sequence_number: Some(rsn),
            earlier_acks: ack_bits,
        })
    }
}

#[cfg(test)]
mod tests {
    use net::udp::seqnum::SequenceNumber;

    use super::AckStatus;

    #[test]
    fn test_ack_control_basic() {
        let mut acks = AckStatus::new();

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
        let mut acks = AckStatus::new();

        let mut seq_num = 65535.into();

        acks.ack(seq_num);

        seq_num += 10.into();
        acks.ack(seq_num);
        assert_eq!(acks.remote_sequence_number.unwrap(), seq_num);
        assert_eq!(acks.earlier_acks, 1 << 9);
    }
}
