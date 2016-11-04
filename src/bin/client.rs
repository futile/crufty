extern crate crufty;

use crufty::util::run_state_machine;
use crufty::application::AppTransition;

fn main() {
    run_state_machine(AppTransition::Startup);
}
