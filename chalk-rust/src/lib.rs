#![recursion_limit = "1024"]
#![feature(conservative_impl_trait)]
#![feature(field_init_shorthand)]
#![feature(pub_restricted)]
#![feature(static_in_const)]

#![allow(dead_code)] // FIXME- while iterating

extern crate chalk_rust_parse;
#[macro_use]
extern crate error_chain;
extern crate ena;
extern crate lalrpop_intern;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

mod cast;
mod errors;
mod fold;
mod ir;
mod lower;
mod solve;
mod zip;

