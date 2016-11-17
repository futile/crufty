extern crate crufty;

use crufty::util;
use crufty::client::{ClientTransition};

fn main() {
    util::run_state_machine(ClientTransition::Startup);
}
