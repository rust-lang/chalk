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
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplData {
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitData {
    pub parameter_kinds: Vec<ParameterKind<Identifier>>, // including the implicit `Self` as param 0
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Lifetime {
    Var(usize),
    ForAll(UniverseIndex),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ApplicationTy {
    pub name: TypeName,
    pub parameters: Vec<Parameter>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ParameterKind<T, L = T> {
    Ty(T),
    Lifetime(L),
}

impl<T> ParameterKind<T, T> {
    pub fn map<OP, U>(self, op: OP) -> ParameterKind<U>
        where OP: FnOnce(T) -> U
    {
        match self {
            ParameterKind::Ty(t) => ParameterKind::Ty(op(t)),
            ParameterKind::Lifetime(t) => ParameterKind::Lifetime(op(t)),
        }
    }
}

impl<T, L> ParameterKind<T, L> {
    pub fn as_ref(&self) -> ParameterKind<&T, &L> {
        match *self {
            ParameterKind::Ty(ref t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(ref l) => ParameterKind::Lifetime(l),
        }
    }

    pub fn ty(self) -> Option<T> {
        match self {
            ParameterKind::Ty(t) => Some(t),
            _ => None,
        }
    }

    pub fn lifetime(self) -> Option<L> {
        match self {
            ParameterKind::Lifetime(t) => Some(t),
            _ => None,
        }
    }
}

pub type Parameter = ParameterKind<Ty, Lifetime>;

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
    pub binders: Vec<ParameterKind<UniverseIndex>>,
}

impl<T> Quantified<T> {
    pub fn map<OP, U>(self, op: OP) -> Quantified<U>
        where OP: FnOnce(T) -> U
    {
        Quantified { value: op(self.value), binders: self.binders }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Constrained<T> {
    pub value: T,
    pub constraints: Vec<Constraint>,
}

impl<T> Constrained<T> {
    pub fn map<OP, U>(self, op: OP) -> Constrained<U>
        where OP: FnOnce(T) -> U
    {
        Constrained { value: op(self.value), constraints: self.constraints }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Goal {
    Quantified(QuantifierKind, ParameterKind<()>, Box<Goal>),
    Implies(Vec<WhereClause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Leaf(WhereClause),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum QuantifierKind {
    ForAll, Exists
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Constraint {
    LifetimeEq(Lifetime, Lifetime),
}

pub mod debug;
mod tls;

pub use self::tls::set_current_program;
pub use self::tls::with_current_program;

