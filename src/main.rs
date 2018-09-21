#![feature(never_type)]
#![feature(const_fn)]
#![feature(drain_filter)]

#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

use nalgebra as na;
use ncollide as nc;

#[macro_use]
pub mod util;

pub mod application;
pub mod systems;
pub mod components;
pub mod game;

fn main() {
    util::run_state_machine(application::AppTransition::Startup);
}
