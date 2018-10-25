#![feature(never_type)]
#![feature(const_fn)]
#![feature(drain_filter)]
#![feature(dbg_macro)]

#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

#[macro_use]
extern crate lazy_static;

use nalgebra as na;
use ncollide2d as nc;

#[macro_use]
pub mod util;

pub mod application;
pub mod components;
pub mod game;
pub mod systems;
pub mod net;
