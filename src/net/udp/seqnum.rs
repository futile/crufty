use std::num::Wrapping;
use std::ops::Sub;
use std::io;
use std::io::prelude::{Write, Read};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

pub type SequenceNumberPrecision = u16;

custom_derive! {
    #[derive(NewtypeAdd, NewtypeAdd(&self), NewtypeAddAssign, PartialOrd, Ord, PartialEq, Eq, Debug, Clone, Copy)]
    pub struct SequenceNumber(Wrapping<SequenceNumberPrecision>);
}

impl SequenceNumber {
    const FIRST_SEQUENCE_NUMBER: SequenceNumberPrecision = 1;

    pub fn new(seq_num: SequenceNumberPrecision) -> SequenceNumber {
        SequenceNumber(Wrapping(seq_num))
    }

    pub fn first() -> SequenceNumber {
        SequenceNumber::new(Self::FIRST_SEQUENCE_NUMBER)
    }

    pub fn write_to_packet<O: ByteOrder, W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u16::<O>((self.0).0)
    }

    pub fn read_from_packet<O: ByteOrder, R: Read>(reader: &mut R) -> io::Result<SequenceNumber> {
        let raw = reader.read_u16::<O>()?;

        Ok(SequenceNumber::new(raw))
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
