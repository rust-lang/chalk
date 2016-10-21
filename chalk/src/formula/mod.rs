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

mod leaf;
pub use self::leaf::*;

mod quant;
pub use self::quant::*;

mod lower;

mod debug;
