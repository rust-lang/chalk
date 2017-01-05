use lalrpop_intern::InternedString;
use std::collections::HashMap;

pub type Identifier = InternedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// From type-name to item-id. Used during lowering only.
    pub type_ids: HashMap<Identifier, ItemId>,

    /// For each struct/trait:
    pub type_kinds: HashMap<ItemId, TypeKind>,

    /// For each impl:
    pub impl_data: HashMap<ItemId, ImplData>,

    /// For each trait:
    pub trait_data: HashMap<ItemId, TraitData>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    pub index: usize
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub name: Identifier,
    pub parameters: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplData {
    pub parameters: usize,
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitData {
    pub parameters: usize, // including the implicit `Self`
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_names: Vec<Identifier>,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Quantified<T> {
    pub value: T,
    pub binders: usize,
}

impl<T> Quantified<T> {
    pub fn map<OP, U>(self, op: OP) -> Quantified<U>
        where OP: FnOnce(T) -> U
    {
        Quantified { value: op(self.value), binders: self.binders }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Goal {
    ForAll(usize, Box<Goal>),
    Exists(usize, Box<Goal>),
    Implies(Vec<WhereClause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Leaf(WhereClause),
}

mod debug;
mod tls;

pub use self::tls::set_current_program;
pub use self::tls::with_current_program;
