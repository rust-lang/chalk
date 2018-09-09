#![recursion_limit = "1024"]
#![cfg_attr(test, feature(test))]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(in_band_lifetimes)]
#![feature(macro_at_most_once_rep)]
#![feature(specialization)]
#![feature(step_trait)]
#![feature(non_modrs_mods)]
#![feature(underscore_imports)]

extern crate chalk_parse;
#[macro_use]
extern crate chalk_macros;
extern crate chalk_engine;
#[macro_use]
extern crate chalk_ir as ir;
extern crate diff;
#[macro_use]
extern crate error_chain;
extern crate itertools;
extern crate lalrpop_intern;
extern crate petgraph;
extern crate stacker;

#[macro_use]
mod test_util;

pub mod rust_ir;

crate mod coherence;
crate mod rules;
pub mod errors;

mod test;

