extern crate crufty;

use crufty::{util, application};

fn main() {
    util::run_state_machine(application::ServerTransition::Startup);
}
