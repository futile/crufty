use std::time::{Instant, Duration};
use std::io;

use smallvec::SmallVec;

use net::udp::{UdpConnection, MessageId, ReceiveEvent};

#[derive(Debug)]
enum ConnectionMode {
    Good {
        start_good: Instant,
        last_tick_good: Instant,
        time_in_next_bad: Duration,
    },
    Bad {
        last_bad_rtt_received: Instant,
        time_until_good: Duration,
    },
}

impl ConnectionMode {
    fn tick_rtt(&mut self, rtt: Duration) {
        use std::cmp::{max, min};

        let good_bad_threshold = Duration::from_millis(250);

        let new: Option<ConnectionMode> = match self {
            &mut ConnectionMode::Good { ref start_good,
                                        ref mut last_tick_good,
                                        ref mut time_in_next_bad } => {
                if rtt > good_bad_threshold {
                    if start_good.elapsed() < Duration::from_secs(10) {
                        *time_in_next_bad = min(*time_in_next_bad * 2, Duration::from_secs(60));
                    }

                    println!("CongestionControl: switching good -> bad");
                    Some(ConnectionMode::Bad {
                        last_bad_rtt_received: Instant::now(),
                        time_until_good: *time_in_next_bad,
                    })
                } else {
                    if last_tick_good.elapsed() >= Duration::from_secs(10) {
                        *time_in_next_bad = max((*time_in_next_bad) / 2, Duration::from_secs(1));
                        *last_tick_good = Instant::now();
                    }
                    None
                }
            }
            &mut ConnectionMode::Bad { ref mut last_bad_rtt_received, time_until_good } => {
                if rtt <= good_bad_threshold {
                    if last_bad_rtt_received.elapsed() >= time_until_good {
                        let now = Instant::now();

                        println!("CongestionControl: switching bad -> good");
                        Some(ConnectionMode::Good {
                            start_good: now,
                            last_tick_good: now,
                            time_in_next_bad: time_until_good,
                        })
                    } else {
                        None
                    }
                } else {
                    *last_bad_rtt_received = Instant::now();
                    None
                }
            }
        };

        if let Some(new_mode) = new {
            *self = new_mode;
        }
    }

    fn get_send_interval(&self) -> Duration {
        match self {
            &ConnectionMode::Good { .. } => Duration::from_secs(1) / 30,
            &ConnectionMode::Bad { .. } => Duration::from_secs(1) / 10,
        }
    }
}

pub struct CongestionControl {
    conn: UdpConnection,
    tracked_rtt: Duration,
    last_send: Instant,
    time_until_next_send: Duration,
    mode: ConnectionMode,
}

impl CongestionControl {
    pub fn new(udp_conn: UdpConnection) -> CongestionControl {
        CongestionControl {
            conn: udp_conn,
            tracked_rtt: Duration::new(0, 0),
            last_send: Instant::now(),
            time_until_next_send: Duration::new(0, 0),
            mode: ConnectionMode::Good {
                start_good: Instant::now(),
                last_tick_good: Instant::now(),
                time_in_next_bad: Duration::new(10, 0),
            },
        }
    }

    pub fn set_packet_timeout_limit(&mut self, new_timeout: Duration) {
        self.conn.set_packet_timeout_limit(new_timeout);
    }

    pub fn send_bytes(&mut self, msg: &[u8]) -> io::Result<MessageId> {
        let now = Instant::now();
        // TODO this isn't 100% correct, since it doesn't allow for
        // long idle periods followed by a burst > max allowed.
        // the scheme allows for this however, so it is doable.
        // it's not essential though.

        // every time we send a packet, we have to check when the next
        // scheduled packet should be sent, and whether we are sending too
        // often.
        self.time_until_next_send = self.time_until_next_send
            .checked_sub(now.duration_since(self.last_send))
            .unwrap_or(Duration::new(0, 0));

        // check if we are sending too often
        if self.time_until_next_send > Duration::from_secs(1) {
            println!("congestion control doesn't allow send yet!");
            return Err(io::Error::new(io::ErrorKind::WouldBlock,
                                      "congestion control: connection overloaded"));
        }

        // sending is ok, updated tracket values
        self.time_until_next_send += self.mode.get_send_interval();
        self.last_send = now;

        // actually send the message
        self.conn.send_bytes(msg)
    }

    pub fn recv_with_timeout<F>(&mut self, timeout: Option<Duration>, mut handler: F)
        where F: FnMut(ReceiveEvent)
    {
        let mut new_rtts: SmallVec<[Duration; 4]> = SmallVec::new();

        self.conn.recv_with_timeout(timeout, |e| {
            if let ReceiveEvent::NewAck(_, rtt) = e {
                new_rtts.push(rtt);
            }

            handler(e);
        });

        for rtt in new_rtts {
            // update average rtt by 10% towards this packet's rtt (see link in udp/mod.rs)
            self.tracked_rtt = (self.tracked_rtt * 9 + rtt) / 10;

            // tick the current mode(good/bad) by the averaged rtt
            // (possibly making it switch modes)
            self.mode.tick_rtt(self.tracked_rtt);
        }
    }

    pub fn check_for_timeouts<F>(&mut self, on_timeout: F)
        where F: FnMut(MessageId)
    {
        self.conn.check_for_timeouts(on_timeout);
    }
}
