#![recursion_limit = "1024"]
#![cfg_attr(feature = "bench", feature(test))]

#[macro_use]
extern crate chalk_macros;

pub mod db;
pub mod error;
pub mod lowering;
pub mod program;
pub mod program_environment;
pub mod query;
