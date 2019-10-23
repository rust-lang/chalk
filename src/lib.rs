#![recursion_limit = "1024"]
#![cfg_attr(feature = "bench", feature(test))]

#[macro_use]
extern crate chalk_macros;

#[macro_use]
extern crate failure;

#[macro_use]
mod test_util;

pub mod db;
pub mod error;
pub mod lowering;
pub mod program;
pub mod program_environment;
pub mod query;

mod test;
