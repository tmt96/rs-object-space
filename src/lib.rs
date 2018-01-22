#![feature(drain_filter)]
#![feature(specialization)]
#![feature(unsize)]
extern crate num_traits;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

pub mod object_space;
mod entry;
