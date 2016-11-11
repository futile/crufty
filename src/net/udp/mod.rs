// this is an implementation following http://gafferongames.com/networking-for-game-programmers/reliability-and-flow-control/

mod seqnum;
mod ackstat;
mod conn;
mod congcontrol;

pub use self::conn::{MessageId, UdpConnection, ReceiveEvent};
pub use self::congcontrol::CongestionControl;

use std::time::Duration;

pub fn ns_to_ms(ns: u32) -> u64 {
    ns as u64 / 1_000_000
}

pub fn dur_to_ms(dur: &Duration) -> u64 {
    dur.as_secs() * 1000 + ns_to_ms(dur.subsec_nanos())
}
