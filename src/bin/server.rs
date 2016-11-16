extern crate crufty;

use crufty::util;
use crufty::server::{ServerTransition};

fn main() {
    util::run_state_machine(ServerTransition::Startup);
}
