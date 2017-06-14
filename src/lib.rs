#![recursion_limit = "1024"]
#![feature(conservative_impl_trait)]

#![allow(dead_code)] // FIXME- while iterating

extern crate chalk_parse;
#[macro_use]
extern crate error_chain;
extern crate ena;
extern crate itertools;
extern crate lalrpop_intern;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

pub mod cast;
pub mod errors;
pub mod fold;
pub mod ir;
pub mod lower;
pub mod overlap;
pub mod solve;
pub mod zip;
