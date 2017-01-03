use lalrpop_intern::InternedString;
use std::collections::HashMap;

pub type Identifier = InternedString;

pub struct Program {
    pub type_kinds: HashMap<InternedString, TypeKind>,
    pub where_clauses: HashMap<ItemId, Vec<WhereClause>>,
    pub trait_defns: Vec<TraitDefn>,
    pub impls: Vec<Impl>,
    pub goals: Vec<WhereClause>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

pub struct TraitDefn {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_defns: Vec<AssocTyDefn>,
}

pub struct AssocTyDefn {
    pub name: Identifier,
}

pub struct Impl {
    pub parameters: Vec<Identifier>,
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<WhereClause>,
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
