use lalrpop_intern::InternedString;
use std::collections::HashMap;

pub type Identifier = InternedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// For each struct/trait:
    pub type_kinds: HashMap<ItemId, TypeKind>,

    /// For each item:
    pub where_clauses: HashMap<ItemId, Vec<WhereClause>>,

    /// For each trait:
    pub assoc_ty_names: HashMap<ItemId, Vec<Identifier>>,

    /// For each impl:
    pub impl_data: HashMap<ItemId, ImplData>,

    pub goals: Vec<WhereClause>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    pub index: usize
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplData {
    pub parameters: Vec<Identifier>,
    pub trait_ref: TraitRef,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssocTyValue {
    pub name: Identifier,
    pub value: Ty,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Var(usize),
    Apply(ApplicationTy),
    Projection(ProjectionTy),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ApplicationTy {
    pub id: ItemId,
    pub args: Vec<Ty>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TraitRef {
    pub trait_id: ItemId,
    pub args: Vec<Ty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WhereClause {
    Implemented(TraitRef),
    NormalizeTo(NormalizeTo),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NormalizeTo {
    pub projection: ProjectionTy,
    pub ty: Ty,
}

mod debug;
mod tls;

pub use self::tls::set_program_in;
pub use self::tls::with_current_program;
