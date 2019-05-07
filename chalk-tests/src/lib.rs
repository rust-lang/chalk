#![recursion_limit = "1024"]
#![cfg_attr(feature = "bench", feature(test))]

#[macro_use]
extern crate failure;

#[macro_use]
mod test_util;

mod test;
