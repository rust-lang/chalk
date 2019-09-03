macro_rules! impl_froms {
    ($e:ident: $($v:ident), *) => {
        $(
            impl From<$v> for $e {
                fn from(it: $v) -> $e {
                    $e::$v(it)
                }
            }
        )*
    }
}

macro_rules! impl_debugs {
    ($($id:ident), *) => {
        $(
            impl std::fmt::Debug for $id {
                fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                    write!(fmt, "{}({:?})", stringify!($id), self.0.index)
                }
            }
        )*
    };
}

use crate::cast::Cast;
use crate::fold::shift::Shift;
use crate::fold::{
    DefaultInferenceFolder, DefaultPlaceholderFolder, DefaultTypeFolder, Fold, FreeVarFolder,
};
use chalk_engine::fallible::*;
use lalrpop_intern::InternedString;
use std::collections::BTreeSet;
use std::iter;
use std::sync::Arc;

extern crate chalk_engine;
extern crate lalrpop_intern;

#[macro_use]
mod macros;

#[macro_use]
pub mod zip;

#[macro_use]
pub mod fold;

pub mod cast;

pub mod could_match;
pub mod debug;
pub mod tls;

pub type Identifier = InternedString;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The set of assumptions we've made so far, and the current number of
/// universal (forall) quantifiers we're within.
pub struct Environment {
    pub clauses: Vec<ProgramClause>,
}

impl Environment {
    pub fn new() -> Arc<Self> {
        Arc::new(Environment { clauses: vec![] })
    }

    pub fn add_clauses<I>(&self, clauses: I) -> Arc<Self>
    where
        I: IntoIterator<Item = ProgramClause>,
    {
        let mut env = self.clone();
        let env_clauses: BTreeSet<_> = env.clauses.into_iter().chain(clauses).collect();
        env.clauses = env_clauses.into_iter().collect();
        Arc::new(env)
    }

    /// Return the set of all in-scope trait ids from the environment.
    /// This is used in "unselected normalization" of projections like
    /// `T::Item`, where the trait is not fully known.
    ///
    /// FIXME -- This is a bit of a hack, because we assume that the
    /// set of in-scope traits is always found in the environment and
    /// has a simple form. This happens to be true, of course.
    pub fn in_scope_trait_ids(&self) -> impl Iterator<Item = TraitId> + '_ {
        self.clauses
            .iter()
            .flat_map(|clause| clause.in_scope_trait_id())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InEnvironment<G> {
    pub environment: Arc<Environment>,
    pub goal: G,
}

impl<G> InEnvironment<G> {
    pub fn new(environment: &Arc<Environment>, goal: G) -> Self {
        InEnvironment {
            environment: environment.clone(),
            goal,
        }
    }

    pub fn map<OP, H>(self, op: OP) -> InEnvironment<H>
    where
        OP: FnOnce(G) -> H,
    {
        InEnvironment {
            environment: self.environment,
            goal: op(self.goal),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeName {
    /// a type like `Vec<T>`
    TypeKindId(TypeKindId),

    /// instantiated form a universally quantified type, e.g., from
    /// `forall<T> { .. }`. Stands in as a representative of "some
    /// unknown type".
    Placeholder(PlaceholderIndex),

    /// an associated type like `Iterator::Item`; see `AssociatedType` for details
    AssociatedType(TypeId),
}

/// An universe index is how a universally quantified parameter is
/// represented when it's binder is moved into the environment.
/// An example chain of transformations would be:
/// `forall<T> { Goal(T) }` (syntatical representation)
/// `forall { Goal(?0) }` (used a DeBruijn index)
/// `Goal(!U1)` (the quantifier was moved to the environment and replaced with a universe index)
/// See https://rust-lang.github.io/rustc-guide/borrow_check/region_inference.html#placeholders-and-universes for more.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseIndex {
    pub counter: usize,
}

impl UniverseIndex {
    pub const ROOT: UniverseIndex = UniverseIndex { counter: 0 };

    pub fn root() -> UniverseIndex {
        Self::ROOT
    }

    pub fn can_see(self, ui: UniverseIndex) -> bool {
        self.counter >= ui.counter
    }

    pub fn next(self) -> UniverseIndex {
        UniverseIndex {
            counter: self.counter + 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructId(pub RawId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitId(pub RawId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImplId(pub RawId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClauseId(pub RawId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub RawId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemId {
    StructId(StructId),
    TraitId(TraitId),
    ImplId(ImplId),
    ClauseId(ClauseId),
    TypeId(TypeId),
}

impl_froms!(ItemId: StructId, TraitId, ImplId, ClauseId, TypeId);
impl_debugs!(ImplId, ClauseId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKindId {
    TypeId(TypeId),
    TraitId(TraitId),
    StructId(StructId),
}

impl TypeKindId {
    pub fn raw_id(&self) -> RawId {
        match self {
            TypeKindId::TypeId(id) => id.0,
            TypeKindId::TraitId(id) => id.0,
            TypeKindId::StructId(id) => id.0,
        }
    }
}

impl_froms!(TypeKindId: TypeId, TraitId, StructId);

impl From<TypeKindId> for ItemId {
    fn from(type_kind_id: TypeKindId) -> ItemId {
        match type_kind_id {
            TypeKindId::TypeId(id) => ItemId::TypeId(id),
            TypeKindId::TraitId(id) => ItemId::TraitId(id),
            TypeKindId::StructId(id) => ItemId::StructId(id),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub struct RawId {
    pub index: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ty {
    /// An "application" type is one that applies the set of type
    /// arguments to some base type. For example, `Vec<u32>` would be
    /// "applying" the parameters `[u32]` to the code type `Vec`.
    /// This type is also used for base types like `u32` (which just apply
    /// an empty list).
    Apply(ApplicationTy),

    /// A "dyn" type is a trait object type created via the "dyn Trait" syntax.
    /// In the chalk parser, the traits that the object represents is parsed as
    /// a QuantifiedInlineBound, and is then changed to a list of where clauses
    /// during lowering.
    ///
    /// See the `Opaque` variant for a discussion about the use of
    /// binders here.
    Dyn(Binders<Vec<QuantifiedWhereClause>>),

    /// An "opaque" type is one that is created via the "impl Trait" syntax.
    /// They are named so because the concrete type implementing the trait
    /// is unknown, and hence the type is opaque to us. The only information
    /// that we know of is that this type implements the traits listed by the
    /// user.
    ///
    /// The "binder" here represents the unknown self type. So, a type like
    /// `impl for<'a> Fn(&'a u32)` would be represented with two-levels of
    /// binder, as "depicted" here:
    ///
    /// ```
    /// exists<type> {
    ///    vec![
    ///        // A QuantifiedWhereClause:
    ///        forall<region> { ^1: Fn(&^2 u32) }
    ///    ]
    /// }
    /// ```
    ///
    /// The outer `exists<type>` binder indicates that there exists
    /// some type that meets the criteria within, but that type is not
    /// known. It is referenced within the type using `^1`, indicating
    /// a bound type with debruijn index 1 (i.e., skipping through one
    /// level of binder).
    Opaque(Binders<Vec<QuantifiedWhereClause>>),

    /// A "projection" type corresponds to an (unnormalized)
    /// projection like `<P0 as Trait<P1..Pn>>::Foo`. Note that the
    /// trait and all its parameters are fully known.
    Projection(ProjectionTy),

    /// A "higher-ranked" type. In the Rust surface syntax, this can
    /// only be a funtion type (e.g., `for<'a> fn(&'a u32)`) or a dyn
    /// type (e.g., `dyn for<'a> SomeTrait<&'a u32>`). However, in
    /// Chalk's representation, we separate out the `for<'a>` part
    /// from the underlying type, so technically we can represent
    /// things like `for<'a> SomeStruct<'a>`, although that has no
    /// meaning in Rust.
    ForAll(Box<QuantifiedTy>),

    /// References the binding at the given depth. The index is a [de
    /// Bruijn index], so it counts back through the in-scope biners,
    /// with 0 being the innermost binder. This is used in impls and
    /// the like. For example, if we had a rule like `for<T> { (T:
    /// Clone) :- (T: Copy) }`, then `T` would be represented as a
    /// `BoundVar(0)` (as the `for` is the innermost binder).
    ///
    /// [de Bruijn index]: https://en.wikipedia.org/wiki/De_Bruijn_index
    BoundVar(usize),

    /// Inference variable defined in the current inference context.
    InferenceVar(InferenceVar),
}

impl Ty {
    /// If this is a `Ty::BoundVar(d)`, returns `Some(d)` else `None`.
    pub fn bound(&self) -> Option<usize> {
        if let Ty::BoundVar(depth) = *self {
            Some(depth)
        } else {
            None
        }
    }

    /// If this is a `Ty::InferenceVar(d)`, returns `Some(d)` else `None`.
    pub fn inference_var(&self) -> Option<InferenceVar> {
        if let Ty::InferenceVar(depth) = *self {
            Some(depth)
        } else {
            None
        }
    }

    pub fn is_projection(&self) -> bool {
        match *self {
            Ty::Projection(..) => true,
            _ => false,
        }
    }

    /// True if this type contains "bound" types/lifetimes, and hence
    /// needs to be shifted across binders. This is a very inefficient
    /// check, intended only for debug assertions, because I am lazy.
    pub fn needs_shift(&self) -> bool {
        *self != self.shifted_in(1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InferenceVar {
    index: u32,
}

impl From<u32> for InferenceVar {
    fn from(index: u32) -> InferenceVar {
        InferenceVar { index }
    }
}

impl InferenceVar {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn to_ty(self) -> Ty {
        Ty::InferenceVar(self)
    }

    pub fn to_lifetime(self) -> Lifetime {
        Lifetime::InferenceVar(self)
    }
}

/// for<'a...'z> X -- all binders are instantiated at once,
/// and we use deBruijn indices within `self.ty`
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuantifiedTy {
    pub num_binders: usize,
    pub ty: Ty,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lifetime {
    /// See Ty::Var(_).
    BoundVar(usize),
    InferenceVar(InferenceVar),
    Placeholder(PlaceholderIndex),
}

impl Lifetime {
    /// If this is a `Lifetime::InferenceVar(d)`, returns `Some(d)` else `None`.
    pub fn inference_var(&self) -> Option<InferenceVar> {
        if let Lifetime::InferenceVar(depth) = *self {
            Some(depth)
        } else {
            None
        }
    }

    /// True if this lifetime is a "bound" lifetime, and hence
    /// needs to be shifted across binders. Meant for debug assertions.
    pub fn needs_shift(&self) -> bool {
        match self {
            Lifetime::BoundVar(_) => true,
            Lifetime::InferenceVar(_) => false,
            Lifetime::Placeholder(_) => false,
        }
    }
}

/// Index of an universally quantified parameter in the environment.
/// Two indexes are required, the one of the universe itself
/// and the relative index inside the universe.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaceholderIndex {
    /// Index *of* the universe.
    pub ui: UniverseIndex,
    /// Index *in* the universe.
    pub idx: usize,
}

impl PlaceholderIndex {
    pub fn to_lifetime(self) -> Lifetime {
        Lifetime::Placeholder(self)
    }

    pub fn to_ty(self) -> Ty {
        Ty::Apply(ApplicationTy {
            name: TypeName::Placeholder(self),
            parameters: vec![],
        })
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ApplicationTy {
    pub name: TypeName,
    pub parameters: Vec<Parameter>,
}

impl ApplicationTy {
    pub fn type_parameters<'a>(&'a self) -> impl Iterator<Item = Ty> + 'a {
        self.parameters.iter().cloned().filter_map(|p| p.ty())
    }

    pub fn first_type_parameter(&self) -> Option<Ty> {
        self.type_parameters().next()
    }

    pub fn len_type_parameters(&self) -> usize {
        self.type_parameters().count()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParameterKind<T, L = T> {
    Ty(T),
    Lifetime(L),
}

impl<T> ParameterKind<T> {
    pub fn into_inner(self) -> T {
        match self {
            ParameterKind::Ty(t) => t,
            ParameterKind::Lifetime(t) => t,
        }
    }

    pub fn map<OP, U>(self, op: OP) -> ParameterKind<U>
    where
        OP: FnOnce(T) -> U,
    {
        match self {
            ParameterKind::Ty(t) => ParameterKind::Ty(op(t)),
            ParameterKind::Lifetime(t) => ParameterKind::Lifetime(op(t)),
        }
    }
}

impl<T, L> ParameterKind<T, L> {
    pub fn assert_ty_ref(&self) -> &T {
        self.as_ref().ty().unwrap()
    }

    pub fn assert_lifetime_ref(&self) -> &L {
        self.as_ref().lifetime().unwrap()
    }

    pub fn as_ref(&self) -> ParameterKind<&T, &L> {
        match *self {
            ParameterKind::Ty(ref t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(ref l) => ParameterKind::Lifetime(l),
        }
    }

    pub fn is_ty(&self) -> bool {
        match self {
            ParameterKind::Ty(_) => true,
            ParameterKind::Lifetime(_) => false,
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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Parameter(pub ParameterKind<Ty, Lifetime>);

impl Parameter {
    pub fn assert_ty_ref(&self) -> &Ty {
        self.as_ref().ty().unwrap()
    }

    pub fn assert_lifetime_ref(&self) -> &Lifetime {
        self.as_ref().lifetime().unwrap()
    }

    pub fn as_ref(&self) -> ParameterKind<&Ty, &Lifetime> {
        match &self.0 {
            ParameterKind::Ty(t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(l) => ParameterKind::Lifetime(l),
        }
    }

    pub fn is_ty(&self) -> bool {
        match self.0 {
            ParameterKind::Ty(_) => true,
            ParameterKind::Lifetime(_) => false,
        }
    }

    pub fn ty(self) -> Option<Ty> {
        match self.0 {
            ParameterKind::Ty(t) => Some(t),
            _ => None,
        }
    }

    pub fn lifetime(self) -> Option<Lifetime> {
        match self.0 {
            ParameterKind::Lifetime(t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectionTy {
    pub associated_ty_id: TypeId,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitRef {
    pub trait_id: TraitId,
    pub parameters: Vec<Parameter>,
}

impl TraitRef {
    pub fn type_parameters<'a>(&'a self) -> impl Iterator<Item = Ty> + 'a {
        self.parameters.iter().cloned().filter_map(|p| p.ty())
    }

    pub fn self_type_parameter(&self) -> Option<Ty> {
        self.type_parameters().next()
    }
}

/// Where clauses that can be written by a Rust programmer.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhereClause {
    Implemented(TraitRef),
    ProjectionEq(ProjectionEq),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WellFormed {
    /// A predicate which is true is some trait ref is well-formed.
    /// For example, given the following trait definitions:
    ///
    /// ```notrust
    /// trait Clone { ... }
    /// trait Copy where Self: Clone { ... }
    /// ```
    ///
    /// then we have the following rule:
    ///
    /// ```notrust
    /// WellFormed(?Self: Copy) :- ?Self: Copy, WellFormed(?Self: Clone)
    /// ```
    Trait(TraitRef),

    /// A predicate which is true is some type is well-formed.
    /// For example, given the following type definition:
    ///
    /// ```notrust
    /// struct Set<K> where K: Hash {
    ///     ...
    /// }
    /// ```
    ///
    /// then we have the following rule: `WellFormedTy(Set<K>) :- Implemented(K: Hash)`.
    Ty(Ty),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FromEnv {
    /// A predicate which enables deriving everything which should be true if we *know* that
    /// some trait ref is well-formed. For example given the above trait definitions, we can use
    /// `FromEnv(T: Copy)` to derive that `T: Clone`, like in:
    ///
    /// ```notrust
    /// forall<T> {
    ///     if (FromEnv(T: Copy)) {
    ///         T: Clone
    ///     }
    /// }
    /// ```
    Trait(TraitRef),

    /// A predicate which enables deriving everything which should be true if we *know* that
    /// some type is well-formed. For example given the above type definition, we can use
    /// `FromEnv(Set<K>)` to derive that `K: Hash`, like in:
    ///
    /// ```notrust
    /// forall<K> {
    ///     if (FromEnv(Set<K>)) {
    ///         K: Hash
    ///     }
    /// }
    /// ```
    Ty(Ty),
}

/// A "domain goal" is a goal that is directly about Rust, rather than a pure
/// logical statement. As much as possible, the Chalk solver should avoid
/// decomposing this enum, and instead treat its values opaquely.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DomainGoal {
    Holds(WhereClause),
    WellFormed(WellFormed),
    FromEnv(FromEnv),

    Normalize(Normalize),

    InScope(TypeKindId),

    /// True if a type is considered to have been "defined" by the current crate. This is true for
    /// a `struct Foo { }` but false for a `#[upstream] struct Foo { }`. However, for fundamental types
    /// like `Box<T>`, it is true if `T` is local.
    IsLocal(Ty),

    /// True if a type is *not* considered to have been "defined" by the current crate. This is
    /// false for a `struct Foo { }` but true for a `#[upstream] struct Foo { }`. However, for
    /// fundamental types like `Box<T>`, it is true if `T` is upstream.
    IsUpstream(Ty),

    /// True if a type and its input types are fully visible, known types. That is, there are no
    /// unknown type parameters anywhere in this type.
    ///
    /// More formally, for each struct S<P0..Pn>:
    /// forall<P0..Pn> {
    ///     IsFullyVisible(S<P0...Pn>) :-
    ///         IsFullyVisible(P0),
    ///         ...
    ///         IsFullyVisible(Pn)
    /// }
    ///
    /// Note that any of these types can have lifetimes in their parameters too, but we only
    /// consider type parameters.
    IsFullyVisible(Ty),

    /// Used to dictate when trait impls are allowed in the current (local) crate based on the
    /// orphan rules.
    ///
    /// `LocalImplAllowed(T: Trait)` is true if the type T is allowed to impl trait Trait in
    /// the current crate. Under the current rules, this is unconditionally true for all types if
    /// the Trait is considered to be "defined" in the current crate. If that is not the case, then
    /// `LocalImplAllowed(T: Trait)` can still be true if `IsLocal(T)` is true.
    LocalImplAllowed(TraitRef),

    /// Used to activate the "compatible modality" rules. Rules that introduce predicates that have
    /// to do with "all compatible universes" should depend on this clause so that they only apply
    /// if this is present.
    ///
    /// (HACK: Having `()` makes some of our macros work better.)
    Compatible(()),

    /// Used to indicate that a given type is in a downstream crate. Downstream crates contain the
    /// current crate at some level of their dependencies.
    ///
    /// Since chalk does not actually see downstream types, this is usually introduced with
    /// implication on a fresh, universally quantified type.
    ///
    /// forall<T> { if (DownstreamType(T)) { /* ... */ } }
    ///
    /// This makes a new type `T` available and makes `DownstreamType(T)` provable for that type.
    DownstreamType(Ty),
}

pub type QuantifiedWhereClause = Binders<WhereClause>;

impl WhereClause {
    /// Turn a where clause into the WF version of it i.e.:
    /// * `Implemented(T: Trait)` maps to `WellFormed(T: Trait)`
    /// * `ProjectionEq(<T as Trait>::Item = Foo)` maps to `WellFormed(<T as Trait>::Item = Foo)`
    /// * any other clause maps to itself
    pub fn into_well_formed_goal(self) -> DomainGoal {
        match self {
            WhereClause::Implemented(trait_ref) => WellFormed::Trait(trait_ref).cast(),
            wc => wc.cast(),
        }
    }

    /// Same as `into_well_formed_goal` but with the `FromEnv` predicate instead of `WellFormed`.
    pub fn into_from_env_goal(self) -> DomainGoal {
        match self {
            WhereClause::Implemented(trait_ref) => FromEnv::Trait(trait_ref).cast(),
            wc => wc.cast(),
        }
    }
}

impl DomainGoal {
    pub fn into_from_env_goal(self) -> DomainGoal {
        match self {
            DomainGoal::Holds(wc) => wc.into_from_env_goal(),
            goal => goal,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A goal that does not involve any logical connectives. Equality is treated
/// specially by the logic (as with most first-order logics), since it interacts
/// with unification etc.
pub enum LeafGoal {
    EqGoal(EqGoal),
    DomainGoal(DomainGoal),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EqGoal {
    pub a: Parameter,
    pub b: Parameter,
}

/// Proves that the given projection **normalizes** to the given
/// type. A projection `T::Foo` normalizes to the type `U` if we can
/// **match it to an impl** and that impl has a `type Foo = V` where
/// `U = V`.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Normalize {
    pub projection: ProjectionTy,
    pub ty: Ty,
}

/// Proves **equality** between a projection `T::Foo` and a type
/// `U`. Equality can be proven via normalization, but we can also
/// prove that `T::Foo = V::Foo` if `T = V` without normalizing.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectionEq {
    pub projection: ProjectionTy,
    pub ty: Ty,
}

/// Indicates that the `value` is universally quantified over `N`
/// parameters of the given kinds, where `N == self.binders.len()`. A
/// variable with depth `i < N` refers to the value at
/// `self.binders[i]`. Variables with depth `>= N` are free.
///
/// (IOW, we use deBruijn indices, where binders are introduced in reverse order
/// of `self.binders`.)
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Binders<T> {
    pub binders: Vec<ParameterKind<()>>,
    pub value: T,
}

impl<T> Binders<T> {
    pub fn map<U, OP>(self, op: OP) -> Binders<U>
    where
        OP: FnOnce(T) -> U,
    {
        let value = op(self.value);
        Binders {
            binders: self.binders,
            value,
        }
    }

    pub fn map_ref<U, OP>(&self, op: OP) -> Binders<U>
    where
        OP: FnOnce(&T) -> U,
    {
        let value = op(&self.value);
        Binders {
            binders: self.binders.clone(),
            value,
        }
    }

    /// Introduces a fresh type variable at the start of the binders and returns new Binders with
    /// the result of the operator function applied.
    ///
    /// forall<?0, ?1> will become forall<?0, ?1, ?2> where ?0 is the fresh variable
    pub fn with_fresh_type_var<U, OP>(self, op: OP) -> Binders<U>
    where
        OP: FnOnce(<T as Fold>::Result, Ty) -> U,
        T: Shift,
    {
        // The new variable is at the front and everything afterwards is shifted up by 1
        let new_var = Ty::BoundVar(0);
        let value = op(self.value.shifted_in(1), new_var);
        Binders {
            binders: iter::once(ParameterKind::Ty(()))
                .chain(self.binders.iter().cloned())
                .collect(),
            value,
        }
    }

    pub fn len(&self) -> usize {
        self.binders.len()
    }
}

/// Allows iterating over a Binders<Vec<T>>, for instance.
/// Each element will include the same set of parameter bounds.
impl<V: IntoIterator> IntoIterator for Binders<V> {
    type Item = Binders<<V as IntoIterator>::Item>;
    type IntoIter = BindersIntoIterator<V>;

    fn into_iter(self) -> Self::IntoIter {
        BindersIntoIterator {
            iter: self.value.into_iter(),
            binders: self.binders,
        }
    }
}

pub struct BindersIntoIterator<V: IntoIterator> {
    iter: <V as IntoIterator>::IntoIter,
    binders: Vec<ParameterKind<()>>,
}

impl<V: IntoIterator> Iterator for BindersIntoIterator<V> {
    type Item = Binders<<V as IntoIterator>::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| Binders {
            binders: self.binders.clone(),
            value: v,
        })
    }
}

/// Represents one clause of the form `consequence :- conditions` where
/// `conditions = cond_1 && cond_2 && ...` is the conjunction of the individual
/// conditions.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramClauseImplication {
    pub consequence: DomainGoal,
    pub conditions: Vec<Goal>,
}

impl ProgramClauseImplication {
    /// See `Environment::in_scope_trait_ids`
    pub fn in_scope_trait_id(&self) -> Option<TraitId> {
        match self.consequence {
            DomainGoal::InScope(TypeKindId::TraitId(trait_id)) => Some(trait_id),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgramClause {
    Implies(ProgramClauseImplication),
    ForAll(Binders<ProgramClauseImplication>),
}

impl ProgramClause {
    /// See `Environment::in_scope_trait_ids`
    pub fn in_scope_trait_id(&self) -> Option<TraitId> {
        match self {
            ProgramClause::Implies(implication) => implication.in_scope_trait_id(),
            ProgramClause::ForAll(binders) => binders.value.in_scope_trait_id(),
        }
    }
}

impl ProgramClause {
    pub fn into_from_env_clause(self) -> ProgramClause {
        match self {
            ProgramClause::Implies(implication) => {
                if implication.conditions.is_empty() {
                    ProgramClause::Implies(ProgramClauseImplication {
                        consequence: implication.consequence.into_from_env_goal(),
                        conditions: vec![],
                    })
                } else {
                    ProgramClause::Implies(implication)
                }
            }
            clause => clause,
        }
    }
}

/// Wraps a "canonicalized item". Items are canonicalized as follows:
///
/// All unresolved existential variables are "renumbered" according to their
/// first appearance; the kind/universe of the variable is recorded in the
/// `binders` field.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Canonical<T> {
    pub value: T,
    pub binders: Vec<ParameterKind<UniverseIndex>>,
}

/// A "universe canonical" value. This is a wrapper around a
/// `Canonical`, indicating that the universes within have been
/// "renumbered" to start from 0 and collapse unimportant
/// distinctions.
///
/// To produce one of these values, use the `u_canonicalize` method.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UCanonical<T> {
    pub canonical: Canonical<T>,
    pub universes: usize,
}

impl<T> UCanonical<T> {
    pub fn is_trivial_substitution(&self, canonical_subst: &Canonical<ConstrainedSubst>) -> bool {
        let subst = &canonical_subst.value.subst;
        assert_eq!(self.canonical.binders.len(), subst.parameters.len());
        subst.is_identity_subst()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum Goal {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    Quantified(QuantifierKind, Binders<Box<Goal>>),
    Implies(Vec<ProgramClause>, Box<Goal>),
    And(Box<Goal>, Box<Goal>),
    Not(Box<Goal>),
    Leaf(LeafGoal),

    /// Indicates something that cannot be proven to be true or false
    /// definitively. This can occur with overflow but also with
    /// unifications of skolemized variables like `forall<X,Y> { X = Y
    /// }`. Of course, that statement is false, as there exist types
    /// X, Y where `X = Y` is not true. But we treat it as "cannot
    /// prove" so that `forall<X,Y> { not { X = Y } }` also winds up
    /// as cannot prove.
    ///
    /// (TOTAL HACK: Having a unit result makes some of our macros work better.)
    CannotProve(()),
}

impl Goal {
    pub fn quantify(self, kind: QuantifierKind, binders: Vec<ParameterKind<()>>) -> Goal {
        Goal::Quantified(
            kind,
            Binders {
                value: Box::new(self),
                binders,
            },
        )
    }

    /// Takes a goal `G` and turns it into `not { G }`
    pub fn negate(self) -> Self {
        Goal::Not(Box::new(self))
    }

    /// Takes a goal `G` and turns it into `compatible { G }`
    pub fn compatible(self) -> Self {
        // compatible { G } desugars into: forall<T> { if (Compatible, DownstreamType(T)) { G } }
        // This activates the compatible modality rules and introduces an anonymous downstream type
        Goal::Quantified(
            QuantifierKind::ForAll,
            Binders {
                value: Box::new(self),
                binders: Vec::new(),
            }
            .with_fresh_type_var(|goal, ty| {
                Box::new(Goal::Implies(
                    vec![
                        DomainGoal::Compatible(()).cast(),
                        DomainGoal::DownstreamType(ty).cast(),
                    ],
                    goal,
                ))
            }),
        )
    }

    pub fn implied_by(self, predicates: Vec<ProgramClause>) -> Goal {
        Goal::Implies(predicates, Box::new(self))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QuantifierKind {
    ForAll,
    Exists,
}

/// A constraint on lifetimes.
///
/// When we search for solutions within the trait system, we essentially ignore
/// lifetime constraints, instead gathering them up to return with our solution
/// for later checking. This allows for decoupling between type and region
/// checking in the compiler.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    LifetimeEq(Lifetime, Lifetime),
}

/// A mapping of inference variables to instantiations thereof.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Substitution {
    /// Map free variable with given index to the value with the same
    /// index. Naturally, the kind of the variable must agree with
    /// the kind of the value.
    pub parameters: Vec<Parameter>,
}

impl Substitution {
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    /// A substitution is an **identity substitution** if it looks
    /// like this
    ///
    /// ```text
    /// ?0 := ?0
    /// ?1 := ?1
    /// ?2 := ?2
    /// ...
    /// ```
    ///
    /// Basically, each value is mapped to a type or lifetime with its
    /// same index.
    pub fn is_identity_subst(&self) -> bool {
        self.parameters
            .iter()
            .zip(0..)
            .all(|(parameter, index)| match parameter.0 {
                ParameterKind::Ty(Ty::BoundVar(depth)) => index == depth,
                ParameterKind::Lifetime(Lifetime::BoundVar(depth)) => index == depth,
                _ => false,
            })
    }
}

impl<'a> DefaultTypeFolder for &'a Substitution {}

impl<'a> DefaultInferenceFolder for &'a Substitution {}

impl<'a> FreeVarFolder for &'a Substitution {
    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        let ty = &self.parameters[depth];
        let ty = ty.assert_ty_ref();
        Ok(ty.shifted_in(binders))
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime> {
        let l = &self.parameters[depth];
        let l = l.assert_lifetime_ref();
        Ok(l.shifted_in(binders))
    }
}

impl<'a> DefaultPlaceholderFolder for &'a Substitution {}

/// Combines a substitution (`subst`) with a set of region constraints
/// (`constraints`). This represents the result of a query; the
/// substitution stores the values for the query's unknown variables,
/// and the constraints represents any region constriants that must
/// additionally be solved.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstrainedSubst {
    pub subst: Substitution,
    pub constraints: Vec<InEnvironment<Constraint>>,
}
