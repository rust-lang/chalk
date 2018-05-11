#![recursion_limit = "1024"]
#![cfg_attr(test, feature(test))]
#![feature(catch_expr)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(in_band_lifetimes)]
#![feature(macro_at_most_once_rep)]
#![feature(macro_vis_matcher)]
#![feature(specialization)]
#![feature(step_trait)]
#![feature(non_modrs_mods)]

extern crate chalk_parse;
#[macro_use]
extern crate chalk_macros;
extern crate chalk_engine;
extern crate diff;
extern crate ena;
#[macro_use]
extern crate error_chain;
extern crate itertools;
extern crate lalrpop_intern;
extern crate petgraph;
extern crate stacker;

#[macro_use]
mod test_util;

#[macro_use]
crate mod fold;

#[macro_use]
crate mod zip;

#[macro_use]
pub mod ir;

crate mod cast;
crate mod coherence;
crate mod rules;
pub mod errors;
pub mod solve;

pub use crate::chalk_engine::fallible;
