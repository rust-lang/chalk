#![recursion_limit = "1024"]
#![feature(static_in_const)]

extern crate chalk_rust_parse;
#[macro_use]
extern crate error_chain;
extern crate ena;
extern crate lalrpop_intern;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref DEBUG_ENABLED: bool = {
        use std::env;
        env::var("CHALK_DEBUG").is_ok()
    };
}

macro_rules! debug {
    ($($t:tt)*) => {
        if *::DEBUG_ENABLED {
            println!($($t)*);
        }
    }
}

#[macro_use]
mod macros;

mod errors;
mod fold;
mod ir;
mod lower;
mod solve;


