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
pub enum TypeName {
    /// a type like `Vec<T>`
    ItemId(ItemId),

    /// skolemized form of a type parameter like `T`
    ForAll(UniverseIndex),

    /// an associated type like `Iterator::Item`; see `AssociatedType` for details
    AssociatedType(AssociatedType),
}

/// Represents an associated item like `Iterator::Item`.  This is used
/// when we have tried to normalize a projection like `T::Item` but
/// couldn't find a better representation.  In that case, we generate
/// an **application type** like `(Iterator::Item)<T>`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssociatedType {
    pub trait_id: ItemId,
    pub name: Identifier,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseIndex {
    pub counter: usize,
}

impl UniverseIndex {
    pub fn root() -> UniverseIndex {
        UniverseIndex { counter: 0 }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    pub index: usize
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplData {
    pub parameter_kinds: Vec<ParameterKind>,
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitData {
    pub parameter_kinds: Vec<ParameterKind>, // including the implicit `Self` as param 0
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
    pub name: TypeName,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ParameterKind {
    Ty(Identifier),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Parameter {
    Ty(Ty),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ProjectionTy {
    pub trait_ref: TraitRef,
    pub name: Identifier,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TraitRef {
    pub trait_id: ItemId,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum WhereClause {
    Implemented(TraitRef),
    Normalize(Normalize),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Normalize {
    pub projection: ProjectionTy,
    pub ty: Ty,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Quantified<T> {
    pub value: T,
    pub binders: Vec<UniverseIndex>,
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

pub mod debug;
mod tls;

pub use self::tls::set_current_program;
pub use self::tls::with_current_program;
