#![recursion_limit = "1024"]
#![cfg_attr(test, feature(test))]
#![feature(crate_visibility_modifier)]
#![feature(in_band_lifetimes)]
#![feature(step_trait)]
#![feature(try_blocks)]

extern crate chalk_parse;
#[macro_use]
extern crate chalk_macros;
extern crate chalk_engine;
#[macro_use]
extern crate chalk_ir;
extern crate chalk_solve;
extern crate diff;
#[macro_use]
extern crate failure;
extern crate itertools;
extern crate lalrpop_intern;
extern crate petgraph;
extern crate stacker;

#[macro_use]
mod test_util;

pub mod rust_ir;

crate mod coherence;
crate mod rules;

pub mod db;
pub mod query;

mod test;
