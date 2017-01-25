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
    pub impl_data: HashMap<ItemId, ImplDatum>,

    /// For each trait:
    pub trait_data: HashMap<ItemId, TraitDatum>,

    /// For each trait:
    pub associated_ty_data: HashMap<ItemId, AssociatedTyDatum>,

    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause>,
}

impl Program {
    pub fn split_projection<'p>(&self, projection: &'p ProjectionTy)
                            -> (&AssociatedTyDatum, &'p [Parameter], &'p [Parameter]) {
        let ProjectionTy { associated_ty_id, ref parameters } = *projection;
        let associated_ty_data = &self.associated_ty_data[&associated_ty_id];
        let trait_datum = &self.trait_data[&associated_ty_data.trait_id];
        let trait_num_params = trait_datum.parameter_kinds.len();
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data, trait_params, other_params)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeName {
    /// a type like `Vec<T>`
    ItemId(ItemId),

    /// skolemized form of a type parameter like `T`
    ForAll(UniverseIndex),

    /// an associated type like `Iterator::Item`; see `AssociatedType` for details
    AssociatedType(ItemId),
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateId {
    pub name: Identifier
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    pub sort: TypeSort,
    pub crate_id: CrateId,
    pub name: Identifier,
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatum {
    pub crate_id: CrateId,
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<WhereClause>,
    pub assoc_ty_values: Vec<AssocTyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum {
    pub crate_id: CrateId,
    pub binders: Binders<StructBoundDatum>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructBoundDatum {
    pub self_ty: ApplicationTy,
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatum {
    pub crate_id: CrateId,
    pub parameter_kinds: Vec<ParameterKind<Identifier>>, // including the implicit `Self` as param 0
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyDatum {
    /// The trait this associated type is defined in.
    pub trait_id: ItemId,

    /// Name of this associated type.
    pub name: Identifier,

    /// Parameters on this associated type, beginning with those from the trait,
    /// but possibly including more.
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,

    /// Where clauses that must hold for the projection be well-formed.
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssocTyValue {
    pub associated_ty_id: ItemId,

    // the for-all encodes add'l binders, beyond those in the impl;
    // free variables reference the enclosing impl
    pub value: Binders<AssocTyValueBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssocTyValueBound {
    /// Type that we normalize to. The X in `type Foo<'a> = X`.
    pub ty: Ty,

    /// Where-clauses that must hold for projection to be valid. The
    /// WC in `type Foo<'a> = X where WC`.
    pub where_clauses: Vec<WhereClause>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    /// References the binding at the given depth (deBruijn index
    /// style). In an inference context (i.e., when solving goals),
    /// free bindings refer into the inference table.
    Var(usize),
    Apply(ApplicationTy),
    Projection(ProjectionTy),
    ForAll(Box<QuantifiedTy>),
}

/// for<'a...'z> X -- all binders are instantiated at once,
/// and we use deBruijn indices within `self.ty`
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct QuantifiedTy {
    pub num_binders: usize,
    pub ty: Ty
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lifetime {
    /// See Ty::Var(_).
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
    pub associated_ty_id: ItemId,
    pub parameters: Vec<Parameter>,
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
pub enum WhereClauseGoal {
    Implemented(TraitRef),
    Normalize(Normalize),
    UnifyTys(Unify<Ty>),
    WellFormed(Ty),
    LocalTo(LocalTo),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LocalTo {
    pub ty: Ty,
    pub crate_id: CrateId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Unify<T> {
    pub a: T,
    pub b: T,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Normalize {
    pub projection: ProjectionTy,
    pub ty: Ty,
}

/// Indicates that the `value` is universally quantified over `N`
/// parameters of the given kinds, where `N == self.binders.len()`. A
/// variable with depth `i < N` refers to the value at
/// `self.binders[i]`. Variables with depth `>= N` are free.
///
/// (IOW, we use deBruijn indices, where binders are introduced in
/// reverse order of `self.binders`.)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Binders<T> {
    pub binders: Vec<ParameterKind<()>>,
    pub value: T,
}

impl<T> Binders<T> {
    pub fn map_ref<U, OP>(&self, op: OP) -> Binders<U>
        where OP: FnOnce(&T) -> U
    {
        let value = op(&self.value);
        Binders {
            binders: self.binders.clone(),
            value: value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramClause {
    pub implication: Binders<ProgramClauseImplication>
}

/// Represents one clause of the form `consequence :- conditions`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramClauseImplication {
    pub consequence: WhereClauseGoal,
    pub conditions: Vec<Goal>,
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
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    Quantified(QuantifierKind, Binders<Box<Goal>>),
    Implies(Vec<WhereClause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Leaf(WhereClauseGoal),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum QuantifierKind {
    ForAll, Exists
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    LifetimeEq(Lifetime, Lifetime),
}

pub mod debug;
mod tls;

pub use self::tls::set_current_program;
pub use self::tls::with_current_program;

