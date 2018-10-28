extern crate crufty;

use crufty::{application, util};

fn main() {
    util::run_state_machine(application::ClientTransition::Startup);
}
