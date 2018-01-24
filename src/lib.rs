#![feature(collections_range)]
extern crate num_traits;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate unreachable;

pub mod object_space;
mod entry;
mod not_nan;
