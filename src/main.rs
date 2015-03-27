extern crate glutin;

#[macro_use]
extern crate glium;

mod util;
mod application;

use util::{run_state_machine};
use application::{AppTransition};

#[allow(dead_code)]
fn main() {
    run_state_machine::<AppTransition>();
}
