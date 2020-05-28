#![recursion_limit = "1024"]
#![cfg_attr(feature = "bench", feature(test))]

pub mod db;
pub mod error;
pub mod interner;
pub mod lowering;
pub mod program;
pub mod program_environment;
pub mod query;
pub mod tls;

use chalk_ir::interner::HasInterner;
use chalk_ir::Binders;
use interner::ChalkIr;

pub use interner::{Identifier, RawId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    FnDef,
    Trait,
    Opaque,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Unit;

impl HasInterner for Unit {
    type Interner = ChalkIr;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub name: Identifier,
    pub binders: Binders<Unit>,
}
