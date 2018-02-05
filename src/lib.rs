#![recursion_limit = "1024"]
#![cfg_attr(test, feature(test))]
#![feature(conservative_impl_trait)]
#![feature(catch_expr)]
#![feature(crate_visibility_modifier)]
#![feature(macro_vis_matcher)]
#![feature(match_default_bindings)]
#![feature(specialization)]
#![feature(step_trait)]
#![feature(universal_impl_trait)]
#![feature(use_nested_groups)]

extern crate chalk_parse;
extern crate diff;
extern crate ena;
#[macro_use]
extern crate error_chain;
extern crate itertools;
extern crate lalrpop_intern;
#[macro_use]
extern crate lazy_static;
extern crate petgraph;
extern crate stacker;

#[macro_use]
mod macros;

#[macro_use]
crate mod fold;

#[macro_use]
crate mod zip;

#[macro_use]
pub mod ir;

crate mod cast;
crate mod coherence;
pub mod errors;
crate mod fallible;
pub mod lower;
pub mod solve;
mod test_util;
