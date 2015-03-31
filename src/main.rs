#![feature(std_misc, thread_sleep)]

extern crate glutin;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

extern crate image;

mod util;
mod application;
mod systems;

use util::{run_state_machine};
use application::{AppTransition};

#[allow(dead_code)]
fn main() {
    run_state_machine(AppTransition::Startup);
}
