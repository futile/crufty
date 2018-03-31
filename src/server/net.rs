use std::net::{UdpSocket, SocketAddr};
use std::time::{Instant, Duration};
use std::io;

use fnv::FnvHashMap;
use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};

use game::PlayerId;
use v2::{self, CContext};
use net::udp::{BasicUdpConnection, ConnectionWrapper, ReliabilityWrapper, CongestionControl};

struct PlayerNetworkInfo {
    wrapper: ConnectionWrapper,
    addr: SocketAddr,
}

pub struct NetworkInterface {
    addrs: FnvHashMap<SocketAddr, PlayerId>,
    infos: FnvHashMap<PlayerId, PlayerNetworkInfo>,
    conn: BasicUdpConnection,
    player_id_counter: u16,
}

impl NetworkInterface {
    pub fn new(local_addr: &SocketAddr) -> NetworkInterface {
        NetworkInterface {
            conn: BasicUdpConnection::new(UdpSocket::bind(local_addr)
                .expect("could not bind socket")),
            infos: Default::default(),
            addrs: Default::default(),
            player_id_counter: 0,
        }
    }

    pub fn perform_send_phase(&mut self, update: &CContext) -> io::Result<()> {
        let ser = v2::serialize_ccontext(&update);

        for (id, info) in &self.infos {
            if !info.wrapper.ready_to_send() {
                continue;
            }

            let (msg_id, payload) = info.wrapper.wrap_payload(&ser)?;
            self.conn.socket.send_to(&payload, info.addr)?;
            println!("update: {} bytes sent to {:?}", ser.len(), id);
        }

        Ok(())
    }

    pub fn perform_receive_phase(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        let deadline = timeout.map(|to| Instant::now() + to);

        // loop: while now() < start + timeout
        loop {
            let now = Instant::now();
            if deadline.map_or(false, |dl| now >= dl) {
                break;
            }

            let remaining = deadline.map(|dl| dl - now);

            // try receiving something
            let (recvd, from_addr) = match self.conn.recv_with_timeout(remaining) {
                Some(res) => res?,
                None => break, // timeout -> we are done for this phase
            };

            // attribute to sender
            // if not existing, create new
            let sender_id = match self.addrs.entry(from_addr) {
                Entry::Occupied(o) => {
                    *o.get()
                }
                Entry::Vacant(v) => {
                    let new_id = PlayerId::from(self.player_id_counter);
                    self.player_id_counter += 1;
                    v.insert(new_id);

                    let new_info = PlayerNetworkInfo {
                        addr: v.key().clone(),
                        wrapper: ConnectionWrapper {
                            cong_control: Some(CongestionControl::new()),
                            reliability: ReliabilityWrapper::new(v.key(), // TODO purge addr in reliability
                                                                 v.key(), // TODO purge addr in reliability
                                                                 Duration::from_secs(1)),
                        },
                    };

                    let prev = self.infos.insert(new_id, new_info);
                    assert!(prev.is_none(), "entry for new id already existing");

                    new_id
                }
            };

            let info = &mut self.infos[&sender_id];
            let data = info.wrapper.unwrap_payload(&recvd, |_| ())?;

            if data.len() > 0 {
                println!("received something from a client(unexpected): {:?}", data);
            }
        }

        for (player_id, info) in &self.infos {
            info.wrapper
                .reliability
                .check_for_timeouts(|msg_id| {
                    println!("msg {:?} to {:?} timed out", msg_id, player_id)
                });
        }

        Ok(())
    }
}
