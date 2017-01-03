use lalrpop_intern::InternedString;
use std::collections::HashMap;

pub type Identifier = InternedString;

pub struct Program {
    /// For each struct/trait:
    pub type_kinds: HashMap<InternedString, TypeKind>,

    /// For each item:
    pub where_clauses: HashMap<ItemId, Vec<WhereClause>>,

    /// For each trait:
    pub assoc_ty_names: HashMap<ItemId, Vec<Identifier>>,

    /// For each impl:
    pub impls: Vec<Impl>,

    pub goals: Vec<WhereClause>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    pub index: usize
}

pub struct TypeKind {
    pub id: ItemId,
    pub sort: TypeSort,
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
}

pub enum TypeSort {
    Struct,
    Trait,
}

pub struct Impl {
    pub id: ItemId,
    pub parameters: Vec<Identifier>,
    pub trait_ref: TraitRef,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

pub struct AssocTyValue {
    pub name: Identifier,
    pub value: Ty,
}

pub enum Ty {
    Var {
        depth: usize,
    },
    Apply {
        name: Identifier,
        args: Vec<Ty>
    },
    Projection {
        proj: ProjectionTy,
    },
}

pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
}

pub struct TraitRef {
    pub trait_name: Identifier,
    pub args: Vec<Ty>,
}

pub enum WhereClause {
    Implemented { trait_ref: TraitRef },
    ProjectionEq { projection: ProjectionTy, ty: Ty },
}
