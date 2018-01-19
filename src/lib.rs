#![feature(drain_filter)]
#![feature(specialization)]
#![feature(unsize)]
extern crate num_traits;
#[macro_use]
extern crate serde;
extern crate serde_json;

pub mod object_space;
mod entry;
mod type_box;
mod serializer;
