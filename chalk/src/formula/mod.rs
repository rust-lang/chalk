//! Compiled formulas. These are based on the user syntax given in
//! `chalk_parse::ast`, but normalized and converted to use debruijn
//! indices.

pub use lalrpop_intern::InternedString;

mod goal;
pub use self::goal::*;

mod clause;
pub use self::clause::*;

mod fold;
pub use self::fold::*;

mod leaf;
pub use self::leaf::*;

mod quant;
pub use self::quant::*;

mod lower;
pub use self::lower::*;

mod debug;

#[macro_use]
mod macros;

