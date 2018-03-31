use std::io;

use net::udp::{MessageId, BasicUdpConnection, NewAckEvent, CongestionControl, ReliabilityWrapper};

pub struct ConnectionWrapper {
    pub cong_control: Option<CongestionControl>,
    pub reliability: ReliabilityWrapper,
}

impl ConnectionWrapper {
    pub fn wrap_payload<'a>(&mut self, payload: &'a [u8]) -> io::Result<(MessageId, Vec<u8>)> {
        self.cong_control
            .map_or(Ok(payload), |cc| cc.wrap_payload(payload))
            .and_then(|pl| self.reliability.wrap_payload(pl))
    }

    pub fn ready_to_send(&self) -> bool {
        self.cong_control.map_or(true, |cc| cc.congestion_status().wait_time().is_none())
    }

    pub fn wrap_and_send_payload(&mut self,
                                 payload: &[u8],
                                 conn: &mut BasicUdpConnection)
                                 -> io::Result<MessageId> {
        self.wrap_payload(payload)
            .and_then(|(msg_id, data)| conn.send(&data).map(|_| msg_id))
    }

    pub fn unwrap_payload<'a, F>(&mut self,
                                 payload: &'a [u8],
                                 new_ack_handler: F)
                                 -> io::Result<&'a [u8]>
        where F: FnMut(NewAckEvent)
    {
        self.reliability.unwrap_payload(payload, |ack| {
            self.cong_control.map(|cc| cc.register_rtt(ack.rtt));
            new_ack_handler(ack);
        })
    }
}
