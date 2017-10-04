#![recursion_limit = "1024"]
#![feature(conservative_impl_trait)]

#![allow(dead_code)] // FIXME- while iterating

extern crate chalk_parse;
#[macro_use]
extern crate error_chain;
extern crate ena;
extern crate itertools;
extern crate petgraph;
extern crate lalrpop_intern;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

#[macro_use]
pub mod fold;

#[macro_use]
pub mod zip;

pub mod cast;
pub mod coherence;
pub mod errors;
pub mod ir;
pub mod lower;
pub mod solve;
