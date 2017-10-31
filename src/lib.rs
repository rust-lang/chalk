#![recursion_limit = "1024"]
#![cfg_attr(test, feature(test))]
#![feature(conservative_impl_trait)]
#![feature(catch_expr)]
#![feature(match_default_bindings)]
#![feature(step_trait)]

extern crate chalk_parse;
#[macro_use]
extern crate error_chain;
extern crate ena;
extern crate itertools;
extern crate petgraph;
extern crate lalrpop_intern;
#[macro_use]
extern crate lazy_static;
extern crate stacker;

#[macro_use]
mod macros;

#[macro_use]
pub mod fold;

#[macro_use]
pub mod zip;

#[macro_use]
pub mod ir;

pub mod cast;
pub mod coherence;
pub mod errors;
pub mod fallible;
pub mod lower;
pub mod solve;
