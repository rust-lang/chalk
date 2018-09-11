#![feature(non_modrs_mods)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]

#[macro_use]
extern crate chalk_macros;
extern crate chalk_engine;
extern crate chalk_ir;
extern crate ena;

pub mod infer;
pub mod solve;
pub mod ext;
