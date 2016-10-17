//! Compiled formulas. These are based on the user syntax given in
//! `chalk_parse::ast`, but normalized and converted to use debruijn
//! indices.

pub use lalrpop_intern::InternedString;

#[macro_use]
mod goal;
pub use self::goal::*;

#[macro_use]
mod clause;
pub use self::clause::*;

mod common;
pub use self::common::*;

