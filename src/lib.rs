#![feature(never_type)]
#![feature(const_fn)]
#![feature(drain_filter)]
#![feature(dbg_macro)]
#![feature(nll)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use nalgebra as na;
use ncollide2d as nc;

#[macro_use]
pub mod util;

pub mod application;
pub mod components;
pub mod game;
pub mod net;
pub mod systems;
