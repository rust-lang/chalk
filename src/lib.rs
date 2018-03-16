#![recursion_limit = "1024"]
#![cfg_attr(test, feature(test))]
#![feature(conservative_impl_trait)]
#![feature(catch_expr)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(dyn_trait)]
#![feature(in_band_lifetimes)]
#![feature(macro_at_most_once_rep)]
#![feature(macro_vis_matcher)]
#![feature(match_default_bindings)]
#![feature(specialization)]
#![feature(step_trait)]
#![feature(underscore_lifetimes)]
#![feature(universal_impl_trait)]

extern crate chalk_parse;
#[macro_use]
extern crate chalk_macros;
extern crate chalk_slg;
extern crate diff;
extern crate ena;
#[macro_use]
extern crate error_chain;
extern crate itertools;
extern crate lalrpop_intern;
extern crate petgraph;
extern crate stacker;

#[macro_use]
crate mod fold;

#[macro_use]
crate mod zip;

#[macro_use]
pub mod ir;

crate mod cast;
crate mod coherence;
pub mod errors;
pub mod lower;
pub mod solve;
mod test_util;

pub use crate::chalk_slg::fallible;
