#![recursion_limit = "1024"]
#![feature(static_in_const)]

extern crate chalk_rust_parse;

#[macro_use]
extern crate error_chain;

extern crate lalrpop_intern;

mod errors;
mod ir;
mod lower;


