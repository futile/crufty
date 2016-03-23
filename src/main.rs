#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

extern crate image;

extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate num;

extern crate hprof;

extern crate clock_ticks;

extern crate ordered_float;

#[macro_use]
mod util;

mod application;
mod systems;
mod components;

use util::run_state_machine;
use application::AppTransition;

fn main() {
    run_state_machine(AppTransition::Startup);
}
