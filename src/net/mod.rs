use std::time::Duration;

use enet::Enet;

mod client;
pub mod serde_impls;
mod server;
mod protocol;

pub use self::client::Client;
pub use self::server::Server;

lazy_static! {
    static ref ENET: Enet = Enet::new().unwrap();
}

const PORT: u16 = 9001;
const RESEND_DURATION: Duration = Duration::from_millis(100);
const UPDATE_CHANNEL_ID: u8 = 1;
