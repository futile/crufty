#![feature(rust_2018_preview)]

#![feature(never_type)]
#![feature(const_fn)]
#![feature(drain_filter)]

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
extern crate toml;

extern crate rand;

extern crate typemap;

#[macro_use]
pub mod util;

pub mod application;
pub mod systems;
pub mod components;
pub mod game;

fn main() {
    util::run_state_machine(application::AppTransition::Startup);
}
