extern crate glutin;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

extern crate image;

extern crate nalgebra as na;

extern crate hprof;

#[macro_use]
mod util;

mod application;
mod systems;
mod components;

use util::{run_state_machine};
use application::{AppTransition};

#[allow(dead_code)]
fn main() {
    run_state_machine(AppTransition::Startup);
}
