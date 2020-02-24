use crate::cast::{Cast, CastTo};
use crate::fold::shift::Shift;
use crate::fold::{Fold, Folder, Subst, SuperFold};
use chalk_derive::{Fold, HasInterner};
use chalk_engine::fallible::*;
use lalrpop_intern::InternedString;
use std::iter;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Void {}

macro_rules! impl_debugs {
    ($($id:ident), *) => {
        $(
            impl<I: Interner> std::fmt::Debug for $id<I> {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                    write!(fmt, "{}({:?})", stringify!($id), self.0)
                }
            }
        )*
    };
}

#[macro_use]
mod macros;

#[macro_use]
pub mod zip;

#[macro_use]
pub mod fold;

pub mod cast;

pub mod interner;
use interner::{HasInterner, Interner, TargetInterner};

pub mod could_match;
pub mod debug;
pub mod tls;

pub type Identifier = InternedString;

#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
/// The set of assumptions we've made so far, and the current number of
/// universal (forall) quantifiers we're within.
pub struct Environment<I: Interner> {
    pub clauses: Vec<ProgramClause<I>>,
}

impl<I: Interner> Environment<I> {
    pub fn new() -> Self {
        Environment { clauses: vec![] }
    }

    pub fn add_clauses<II>(&self, clauses: II) -> Self
    where
        II: IntoIterator<Item = ProgramClause<I>>,
    {
        let mut env = self.clone();
        env.clauses = env.clauses.into_iter().chain(clauses).collect();
        env
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold)]
pub struct InEnvironment<G: HasInterner> {
    pub environment: Environment<G::Interner>,
    pub goal: G,
}

impl<G: HasInterner> InEnvironment<G> {
    pub fn new(environment: &Environment<G::Interner>, goal: G) -> Self {
        InEnvironment {
            environment: environment.clone(),
            goal,
        }
    }

    pub fn map<OP, H>(self, op: OP) -> InEnvironment<H>
    where
        OP: FnOnce(G) -> H,
        H: HasInterner<Interner = G::Interner>,
    {
        InEnvironment {
            environment: self.environment,
            goal: op(self.goal),
        }
    }
}

impl<G: HasInterner> HasInterner for InEnvironment<G> {
    type Interner = G::Interner;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Fold)]
pub enum TypeName<I: Interner> {
    /// a type like `Vec<T>`
    Struct(StructId<I>),

    /// an associated type like `Iterator::Item`; see `AssociatedType` for details
    AssociatedType(AssocTypeId<I>),

    /// This can be used to represent an error, e.g. during name resolution of a type.
    /// Chalk itself will not produce this, just pass it through when given.
    Error,
}

/// An universe index is how a universally quantified parameter is
/// represented when it's binder is moved into the environment.
/// An example chain of transformations would be:
/// `forall<T> { Goal(T) }` (syntactical representation)
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
pub struct StructId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImplId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClauseId<I: Interner>(pub I::DefId);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssocTypeId<I: Interner>(pub I::DefId);

impl_debugs!(ImplId, ClauseId);

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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct Ty<I: Interner> {
    interned: I::InternedType,
}

impl<I: Interner> Ty<I> {
    pub fn new(data: impl CastTo<TyData<I>>) -> Self {
        Ty {
            interned: I::intern_ty(data.cast()),
        }
    }

    pub fn data(&self) -> &TyData<I> {
        I::ty_data(&self.interned)
    }

    pub fn from_env(&self) -> FromEnv<I> {
        FromEnv::Ty(self.clone())
    }

    pub fn well_formed(&self) -> WellFormed<I> {
        WellFormed::Ty(self.clone())
    }

    /// If this is a `TyData::BoundVar(d)`, returns `Some(d)` else `None`.
    pub fn bound(&self) -> Option<usize> {
        if let TyData::BoundVar(depth) = self.data() {
            Some(*depth)
        } else {
            None
        }
    }

    /// If this is a `TyData::InferenceVar(d)`, returns `Some(d)` else `None`.
    pub fn inference_var(&self) -> Option<InferenceVar> {
        if let TyData::InferenceVar(depth) = self.data() {
            Some(*depth)
        } else {
            None
        }
    }

    pub fn is_alias(&self) -> bool {
        match self.data() {
            TyData::Alias(..) => true,
            _ => false,
        }
    }

    /// True if this type contains "bound" types/lifetimes, and hence
    /// needs to be shifted across binders. This is a very inefficient
    /// check, intended only for debug assertions, because I am lazy.
    pub fn needs_shift(&self) -> bool {
        let ty = self.clone();
        ty != ty.shifted_in(1)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, HasInterner)]
pub enum TyData<I: Interner> {
    /// An "application" type is one that applies the set of type
    /// arguments to some base type. For example, `Vec<u32>` would be
    /// "applying" the parameters `[u32]` to the code type `Vec`.
    /// This type is also used for base types like `u32` (which just apply
    /// an empty list).
    Apply(ApplicationTy<I>),

    /// instantiated form a universally quantified type, e.g., from
    /// `forall<T> { .. }`. Stands in as a representative of "some
    /// unknown type".
    Placeholder(PlaceholderIndex),

    /// A "dyn" type is a trait object type created via the "dyn Trait" syntax.
    /// In the chalk parser, the traits that the object represents is parsed as
    /// a QuantifiedInlineBound, and is then changed to a list of where clauses
    /// during lowering.
    ///
    /// See the `Opaque` variant for a discussion about the use of
    /// binders here.
    Dyn(DynTy<I>),

    /// An "alias" type represents some form of type alias, such as:
    /// - An associated type projection like `<T as Iterator>::Item`
    /// - `impl Trait` types
    /// - Named type aliases like `type Foo<X> = Vec<X>`
    Alias(AliasTy<I>),

    /// A function type such as `for<'a> fn(&'a u32)`.
    /// Note that "higher-ranked" types (starting with `for<>`) are either
    /// function types or dyn types, and do not appear otherwise in Rust
    /// surface syntax.
    Function(Fn<I>),

    /// References the binding at the given depth. The index is a [de
    /// Bruijn index], so it counts back through the in-scope binders,
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

impl<I: Interner> TyData<I> {
    pub fn intern(self) -> Ty<I> {
        Ty::new(self)
    }
}

/// A "DynTy" could be either a `dyn Trait` or an (opaque) `impl
/// Trait`. Both of them are conceptually very related to a
/// "existential type" of the form `exists<T> { T: Trait }`. The
/// `DynTy` type represents those bounds.
///
/// The "binder" here represents the unknown self type. So, a type like
/// `impl for<'a> Fn(&'a u32)` would be represented with two-levels of
/// binder, as "depicted" here:
///
/// ```notrust
/// exists<type> {
///    vec![
///        // A QuantifiedWhereClause:
///        forall<region> { ^1: Fn(&^0 u32) }
///    ]
/// }
/// ```
///
/// The outer `exists<type>` binder indicates that there exists
/// some type that meets the criteria within, but that type is not
/// known. It is referenced within the type using `^1`, indicating
/// a bound type with debruijn index 1 (i.e., skipping through one
/// level of binder).
#[derive(Clone, PartialEq, Eq, Hash, Fold)]
pub struct DynTy<I: Interner> {
    pub bounds: Binders<Vec<QuantifiedWhereClause<I>>>,
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

    pub fn to_ty<I: Interner>(self) -> Ty<I> {
        TyData::<I>::InferenceVar(self).intern()
    }

    pub fn to_lifetime<I: Interner>(self) -> Lifetime<I> {
        LifetimeData::<I>::InferenceVar(self).intern()
    }
}

/// for<'a...'z> X -- all binders are instantiated at once,
/// and we use deBruijn indices within `self.ty`
#[derive(Clone, PartialEq, Eq, Hash, HasInterner)]
pub struct Fn<I: Interner> {
    pub num_binders: usize,
    pub parameters: Vec<Parameter<I>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct Lifetime<I: Interner> {
    interned: I::InternedLifetime,
}

impl<I: Interner> Lifetime<I> {
    pub fn new(data: impl CastTo<LifetimeData<I>>) -> Self {
        Lifetime {
            interned: I::intern_lifetime(data.cast()),
        }
    }

    pub fn data(&self) -> &LifetimeData<I> {
        I::lifetime_data(&self.interned)
    }

    /// If this is a `Lifetime::InferenceVar(d)`, returns `Some(d)` else `None`.
    pub fn inference_var(&self) -> Option<InferenceVar> {
        if let LifetimeData::InferenceVar(depth) = self.data() {
            Some(*depth)
        } else {
            None
        }
    }

    /// True if this lifetime is a "bound" lifetime, and hence
    /// needs to be shifted across binders. Meant for debug assertions.
    pub fn needs_shift(&self) -> bool {
        match self.data() {
            LifetimeData::BoundVar(_) => true,
            LifetimeData::InferenceVar(_) => false,
            LifetimeData::Placeholder(_) => false,
            LifetimeData::Phantom(..) => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LifetimeData<I: Interner> {
    /// See TyData::Var(_).
    BoundVar(usize),
    InferenceVar(InferenceVar),
    Placeholder(PlaceholderIndex),
    Phantom(Void, PhantomData<I>),
}

impl<I: Interner> LifetimeData<I> {
    pub fn intern(self) -> Lifetime<I> {
        Lifetime::new(self)
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
    pub fn to_lifetime<I: Interner>(self) -> Lifetime<I> {
        LifetimeData::<I>::Placeholder(self).intern()
    }

    pub fn to_ty<I: Interner>(self) -> Ty<I> {
        let data: TyData<I> = TyData::Placeholder(self);
        data.intern()
    }
}

// Fold derive intentionally omitted, folded through Ty
#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub struct ApplicationTy<I: Interner> {
    pub name: TypeName<I>,
    pub substitution: Substitution<I>,
}

impl<I: Interner> ApplicationTy<I> {
    pub fn intern(self) -> Ty<I> {
        Ty::new(self)
    }

    pub fn type_parameters<'a>(&'a self) -> impl Iterator<Item = Ty<I>> + 'a {
        self.substitution.iter().filter_map(|p| p.ty()).cloned()
    }

    pub fn first_type_parameter(&self) -> Option<Ty<I>> {
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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
pub struct Parameter<I: Interner>(I::InternedParameter);

impl<I: Interner> Parameter<I> {
    pub fn new(data: ParameterData<I>) -> Self {
        let interned = I::intern_parameter(data);
        Parameter(interned)
    }

    pub fn data(&self) -> &ParameterData<I> {
        I::parameter_data(&self.0)
    }

    pub fn assert_ty_ref(&self) -> &Ty<I> {
        self.as_ref().ty().unwrap()
    }

    pub fn assert_lifetime_ref(&self) -> &Lifetime<I> {
        self.as_ref().lifetime().unwrap()
    }

    pub fn as_ref(&self) -> ParameterKind<&Ty<I>, &Lifetime<I>> {
        match self.data() {
            ParameterKind::Ty(t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(l) => ParameterKind::Lifetime(l),
        }
    }

    pub fn is_ty(&self) -> bool {
        match self.data() {
            ParameterKind::Ty(_) => true,
            ParameterKind::Lifetime(_) => false,
        }
    }

    pub fn ty(&self) -> Option<&Ty<I>> {
        match self.data() {
            ParameterKind::Ty(t) => Some(t),
            _ => None,
        }
    }

    pub fn lifetime(&self) -> Option<&Lifetime<I>> {
        match self.data() {
            ParameterKind::Lifetime(t) => Some(t),
            _ => None,
        }
    }
}

#[allow(type_alias_bounds)]
pub type ParameterData<I: Interner> = ParameterKind<Ty<I>, Lifetime<I>>;

impl<I: Interner> ParameterData<I> {
    pub fn intern(self) -> Parameter<I> {
        Parameter::new(self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub struct AliasTy<I: Interner> {
    pub associated_ty_id: AssocTypeId<I>,
    pub substitution: Substitution<I>,
}

impl<I: Interner> AliasTy<I> {
    pub fn intern(self) -> Ty<I> {
        Ty::new(self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub struct TraitRef<I: Interner> {
    pub trait_id: TraitId<I>,
    pub substitution: Substitution<I>,
}

impl<I: Interner> TraitRef<I> {
    pub fn type_parameters<'a>(&'a self) -> impl Iterator<Item = Ty<I>> + 'a {
        self.substitution.iter().filter_map(|p| p.ty()).cloned()
    }

    pub fn self_type_parameter(&self) -> Ty<I> {
        self.type_parameters().next().unwrap()
    }

    pub fn from_env(self) -> FromEnv<I> {
        FromEnv::Trait(self)
    }

    pub fn well_formed(self) -> WellFormed<I> {
        WellFormed::Trait(self)
    }
}

/// Where clauses that can be written by a Rust programmer.
#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub enum WhereClause<I: Interner> {
    Implemented(TraitRef<I>),
    AliasEq(AliasEq<I>),
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub enum WellFormed<I: Interner> {
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
    Trait(TraitRef<I>),

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
    Ty(Ty<I>),
}

#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub enum FromEnv<I: Interner> {
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
    Trait(TraitRef<I>),

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
    Ty(Ty<I>),
}

/// A "domain goal" is a goal that is directly about Rust, rather than a pure
/// logical statement. As much as possible, the Chalk solver should avoid
/// decomposing this enum, and instead treat its values opaquely.
#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub enum DomainGoal<I: Interner> {
    Holds(WhereClause<I>),

    WellFormed(WellFormed<I>),

    FromEnv(FromEnv<I>),

    Normalize(Normalize<I>),

    /// True if a type is considered to have been "defined" by the current crate. This is true for
    /// a `struct Foo { }` but false for a `#[upstream] struct Foo { }`. However, for fundamental types
    /// like `Box<T>`, it is true if `T` is local.
    IsLocal(Ty<I>),

    /// True if a type is *not* considered to have been "defined" by the current crate. This is
    /// false for a `struct Foo { }` but true for a `#[upstream] struct Foo { }`. However, for
    /// fundamental types like `Box<T>`, it is true if `T` is upstream.
    IsUpstream(Ty<I>),

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
    IsFullyVisible(Ty<I>),

    /// Used to dictate when trait impls are allowed in the current (local) crate based on the
    /// orphan rules.
    ///
    /// `LocalImplAllowed(T: Trait)` is true if the type T is allowed to impl trait Trait in
    /// the current crate. Under the current rules, this is unconditionally true for all types if
    /// the Trait is considered to be "defined" in the current crate. If that is not the case, then
    /// `LocalImplAllowed(T: Trait)` can still be true if `IsLocal(T)` is true.
    LocalImplAllowed(TraitRef<I>),

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
    DownstreamType(Ty<I>),
}

pub type QuantifiedWhereClause<I> = Binders<WhereClause<I>>;

impl<I: Interner> WhereClause<I> {
    /// Turn a where clause into the WF version of it i.e.:
    /// * `Implemented(T: Trait)` maps to `WellFormed(T: Trait)`
    /// * `AliasEq(<T as Trait>::Item = Foo)` maps to `WellFormed(<T as Trait>::Item = Foo)`
    /// * any other clause maps to itself
    pub fn into_well_formed_goal(self) -> DomainGoal<I> {
        match self {
            WhereClause::Implemented(trait_ref) => WellFormed::Trait(trait_ref).cast(),
            wc => wc.cast(),
        }
    }

    /// Same as `into_well_formed_goal` but with the `FromEnv` predicate instead of `WellFormed`.
    pub fn into_from_env_goal(self) -> DomainGoal<I> {
        match self {
            WhereClause::Implemented(trait_ref) => FromEnv::Trait(trait_ref).cast(),
            wc => wc.cast(),
        }
    }
}

impl<I: Interner> QuantifiedWhereClause<I> {
    /// As with `WhereClause::into_well_formed_goal`, but for a
    /// quantified where clause. For example, `forall<T> {
    /// Implemented(T: Trait)}` would map to `forall<T> {
    /// WellFormed(T: Trait) }`.
    pub fn into_well_formed_goal(self) -> Binders<DomainGoal<I>> {
        self.map(|wc| wc.into_well_formed_goal())
    }

    /// As with `WhereClause::into_from_env_goal`, but mapped over any
    /// binders. For example, `forall<T> {
    /// Implemented(T: Trait)}` would map to `forall<T> {
    /// FromEnv(T: Trait) }`.
    pub fn into_from_env_goal(self) -> Binders<DomainGoal<I>> {
        self.map(|wc| wc.into_from_env_goal())
    }
}

impl<I: Interner> DomainGoal<I> {
    /// Convert `Implemented(...)` into `FromEnv(...)`, but leave other
    /// goals unchanged.
    pub fn into_from_env_goal(self) -> DomainGoal<I> {
        match self {
            DomainGoal::Holds(wc) => wc.into_from_env_goal(),
            goal => goal,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Fold)]
pub struct EqGoal<I: Interner> {
    pub a: Parameter<I>,
    pub b: Parameter<I>,
}

/// Proves that the given type alias **normalizes** to the given
/// type. A projection `T::Foo` normalizes to the type `U` if we can
/// **match it to an impl** and that impl has a `type Foo = V` where
/// `U = V`.
#[derive(Clone, PartialEq, Eq, Hash, Fold)]
pub struct Normalize<I: Interner> {
    pub alias: AliasTy<I>,
    pub ty: Ty<I>,
}

/// Proves **equality** between a projection `T::Foo` and a type
/// `U`. Equality can be proven via normalization, but we can also
/// prove that `T::Foo = V::Foo` if `T = V` without normalizing.
#[derive(Clone, PartialEq, Eq, Hash, Fold)]
pub struct AliasEq<I: Interner> {
    pub alias: AliasTy<I>,
    pub ty: Ty<I>,
}

impl<I: Interner> HasInterner for AliasEq<I> {
    type Interner = I;
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

impl<T: HasInterner> HasInterner for Binders<T> {
    type Interner = T::Interner;
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

    pub fn map_ref<'a, U, OP>(&'a self, op: OP) -> Binders<U>
    where
        OP: FnOnce(&'a T) -> U,
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
    pub fn with_fresh_type_var<U, I>(
        self,
        op: impl FnOnce(<T as Fold<I, I>>::Result, Ty<I>) -> U,
    ) -> Binders<U>
    where
        I: Interner,
        T: Shift<I>,
    {
        // The new variable is at the front and everything afterwards is shifted up by 1
        let new_var = TyData::<I>::BoundVar(0).intern();
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

impl<T, I> Binders<T>
where
    T: Fold<I, I> + HasInterner<Interner = I>,
    I: Interner,
{
    /// Substitute `parameters` for the variables introduced by these
    /// binders. So if the binders represent (e.g.) `<X, Y> { T }` and
    /// parameters is the slice `[A, B]`, then returns `[X => A, Y =>
    /// B] T`.
    pub fn substitute(&self, parameters: &(impl AsParameters<I> + ?Sized)) -> T::Result {
        let parameters = parameters.as_parameters();
        assert_eq!(self.binders.len(), parameters.len());
        Subst::apply(parameters, &self.value)
    }
}

/// Allows iterating over a `&Binders<Vec<T>>`, for instance. Each
/// element will be a `Binders<&T>`.
impl<'a, V> IntoIterator for &'a Binders<V>
where
    V: HasInterner,
    &'a V: IntoIterator,
{
    type Item = Binders<<&'a V as IntoIterator>::Item>;
    type IntoIter = BindersIntoIterator<&'a V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map_ref(|r| r).into_iter()
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
#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub struct ProgramClauseImplication<I: Interner> {
    pub consequence: DomainGoal<I>,
    pub conditions: Goals<I>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ProgramClause<I: Interner> {
    Implies(ProgramClauseImplication<I>),
    ForAll(Binders<ProgramClauseImplication<I>>),
}

impl<I: Interner> ProgramClause<I> {
    pub fn into_from_env_clause(self) -> ProgramClause<I> {
        match self {
            ProgramClause::Implies(implication) => {
                if implication.conditions.is_empty() {
                    ProgramClause::Implies(ProgramClauseImplication {
                        consequence: implication.consequence.into_from_env_goal(),
                        conditions: Goals::new(),
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
    pub fn is_trivial_substitution<I: Interner>(
        &self,
        canonical_subst: &Canonical<AnswerSubst<I>>,
    ) -> bool {
        let subst = &canonical_subst.value.subst;
        assert_eq!(self.canonical.binders.len(), subst.parameters().len());
        subst.is_identity_subst()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, HasInterner)]
/// A list of goals.
pub struct Goals<I: Interner> {
    goals: I::InternedGoals,
}

impl<I: Interner> Goals<I> {
    pub fn new() -> Self {
        Self::from(None::<Goal<I>>)
    }

    pub fn from(goals: impl IntoIterator<Item = impl CastTo<Goal<I>>>) -> Self {
        use crate::cast::Caster;
        Goals {
            goals: I::intern_goals(goals.into_iter().casted()),
        }
    }

    pub fn from_fallible<E>(
        goals: impl IntoIterator<Item = Result<impl CastTo<Goal<I>>, E>>,
    ) -> Result<Self, E> {
        use crate::cast::Caster;
        let goals = goals
            .into_iter()
            .casted()
            .collect::<Result<Vec<Goal<I>>, _>>()?;
        Ok(Goals::from(goals))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Goal<I>> {
        self.as_slice().iter()
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn as_slice(&self) -> &[Goal<I>] {
        I::goals_data(&self.goals)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, HasInterner)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub struct Goal<I: Interner> {
    interned: I::InternedGoal,
}

impl<I: Interner> Goal<I> {
    pub fn new(interned: GoalData<I>) -> Self {
        let interned = I::intern_goal(interned);
        Self { interned }
    }

    pub fn data(&self) -> &GoalData<I> {
        I::goal_data(&self.interned)
    }

    pub fn quantify(self, kind: QuantifierKind, binders: Vec<ParameterKind<()>>) -> Goal<I> {
        GoalData::Quantified(
            kind,
            Binders {
                value: self,
                binders,
            },
        )
        .intern()
    }

    /// Takes a goal `G` and turns it into `not { G }`
    pub fn negate(self) -> Self {
        GoalData::Not(self).intern()
    }

    /// Takes a goal `G` and turns it into `compatible { G }`
    pub fn compatible(self) -> Self {
        // compatible { G } desugars into: forall<T> { if (Compatible, DownstreamType(T)) { G } }
        // This activates the compatible modality rules and introduces an anonymous downstream type
        GoalData::Quantified(
            QuantifierKind::ForAll,
            Binders {
                value: self,
                binders: Vec::new(),
            }
            .with_fresh_type_var(|goal, ty| {
                GoalData::Implies(
                    vec![
                        DomainGoal::Compatible(()).cast(),
                        DomainGoal::DownstreamType(ty).cast(),
                    ],
                    goal,
                )
                .intern()
            }),
        )
        .intern()
    }

    pub fn implied_by(self, predicates: Vec<ProgramClause<I>>) -> Goal<I> {
        GoalData::Implies(predicates, self).intern()
    }

    /// True if this goal is "trivially true" -- i.e., no work is
    /// required to prove it.
    pub fn is_trivially_true(&self) -> bool {
        match self.data() {
            GoalData::All(goals) => goals.is_empty(),
            _ => false,
        }
    }
}

impl<I> std::iter::FromIterator<Goal<I>> for Box<Goal<I>>
where
    I: Interner,
{
    fn from_iter<II>(iter: II) -> Self
    where
        II: IntoIterator<Item = Goal<I>>,
    {
        Box::new(iter.into_iter().collect())
    }
}

impl<I> std::iter::FromIterator<Goal<I>> for Goal<I>
where
    I: Interner,
{
    fn from_iter<II>(iter: II) -> Self
    where
        II: IntoIterator<Item = Goal<I>>,
    {
        let mut iter = iter.into_iter();
        if let Some(goal0) = iter.next() {
            if let Some(goal1) = iter.next() {
                // More than one goal to prove
                let goals = Goals::from(Some(goal0).into_iter().chain(Some(goal1)).chain(iter));
                GoalData::All(goals).intern()
            } else {
                // One goal to prove
                goal0
            }
        } else {
            // No goals to prove, always true
            GoalData::All(Goals::new()).intern()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, HasInterner)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum GoalData<I: Interner> {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    Quantified(QuantifierKind, Binders<Goal<I>>),
    Implies(Vec<ProgramClause<I>>, Goal<I>),
    All(Goals<I>),
    Not(Goal<I>),

    /// Make two things equal; the rules for doing so are well known to the logic
    EqGoal(EqGoal<I>),

    /// A "domain goal" indicates some base sort of goal that can be
    /// proven via program clauses
    DomainGoal(DomainGoal<I>),

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

impl<I: Interner> GoalData<I> {
    pub fn intern(self) -> Goal<I> {
        Goal::new(self)
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
#[derive(Clone, PartialEq, Eq, Hash, Fold, HasInterner)]
pub enum Constraint<I: Interner> {
    LifetimeEq(Lifetime<I>, Lifetime<I>),
}

/// A mapping of inference variables to instantiations thereof.
#[derive(Clone, PartialEq, Eq, Hash, HasInterner)]
pub struct Substitution<I: Interner> {
    /// Map free variable with given index to the value with the same
    /// index. Naturally, the kind of the variable must agree with
    /// the kind of the value.
    parameters: I::InternedSubstitution,
}

impl<I: Interner> Substitution<I> {
    pub fn from(parameters: impl IntoIterator<Item = impl CastTo<Parameter<I>>>) -> Self {
        Self::from_fallible(
            parameters
                .into_iter()
                .map(|p| -> Result<Parameter<I>, ()> { Ok(p.cast()) }),
        )
        .unwrap()
    }

    pub fn from_fallible<E>(
        parameters: impl IntoIterator<Item = Result<impl CastTo<Parameter<I>>, E>>,
    ) -> Result<Self, E> {
        use crate::cast::Caster;
        Ok(Substitution {
            parameters: I::intern_substitution(parameters.into_iter().casted())?,
        })
    }

    /// Index into the list of parameters
    pub fn at(&self, index: usize) -> &Parameter<I> {
        &self.parameters()[index]
    }

    pub fn from1(parameter: impl CastTo<Parameter<I>>) -> Self {
        Self::from(Some(parameter))
    }

    pub fn empty() -> Self {
        Self::from(None::<Parameter<I>>)
    }

    pub fn is_empty(&self) -> bool {
        self.parameters().is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Parameter<I>> {
        self.parameters().iter()
    }

    pub fn parameters(&self) -> &[Parameter<I>] {
        I::substitution_data(&self.parameters)
    }

    pub fn len(&self) -> usize {
        self.parameters().len()
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
        self.iter()
            .zip(0..)
            .all(|(parameter, index)| match parameter.data() {
                ParameterKind::Ty(ty) => match ty.data() {
                    TyData::BoundVar(depth) => index == *depth,
                    _ => false,
                },
                ParameterKind::Lifetime(lifetime) => match lifetime.data() {
                    LifetimeData::BoundVar(depth) => index == *depth,
                    _ => false,
                },
            })
    }
}

pub trait AsParameters<I: Interner> {
    fn as_parameters(&self) -> &[Parameter<I>];
}

impl<I: Interner> AsParameters<I> for Substitution<I> {
    fn as_parameters(&self) -> &[Parameter<I>] {
        self.parameters()
    }
}

impl<I: Interner> AsParameters<I> for [Parameter<I>] {
    fn as_parameters(&self) -> &[Parameter<I>] {
        self
    }
}

impl<I: Interner> AsParameters<I> for [Parameter<I>; 1] {
    fn as_parameters(&self) -> &[Parameter<I>] {
        self
    }
}

impl<I: Interner> AsParameters<I> for Vec<Parameter<I>> {
    fn as_parameters(&self) -> &[Parameter<I>] {
        self
    }
}

impl<T, I: Interner> AsParameters<I> for &T
where
    T: ?Sized + AsParameters<I>,
{
    fn as_parameters(&self) -> &[Parameter<I>] {
        T::as_parameters(self)
    }
}

impl<'me, I> std::iter::IntoIterator for &'me Substitution<I>
where
    I: Interner,
{
    type IntoIter = std::slice::Iter<'me, Parameter<I>>;
    type Item = &'me Parameter<I>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<I: Interner> Folder<I> for &Substitution<I> {
    fn as_dyn(&mut self) -> &mut dyn Folder<I> {
        self
    }

    fn fold_free_var_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty<I>> {
        let ty = self.at(depth);
        let ty = ty.assert_ty_ref();
        Ok(ty.shifted_in(binders))
    }

    fn fold_free_var_lifetime(&mut self, depth: usize, binders: usize) -> Fallible<Lifetime<I>> {
        let l = self.at(depth);
        let l = l.assert_lifetime_ref();
        Ok(l.shifted_in(binders))
    }
}

/// Combines a substitution (`subst`) with a set of region constraints
/// (`constraints`). This represents the result of a query; the
/// substitution stores the values for the query's unknown variables,
/// and the constraints represents any region constraints that must
/// additionally be solved.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, HasInterner)]
pub struct ConstrainedSubst<I: Interner> {
    pub subst: Substitution<I>, /* NB: The `is_trivial` routine relies on the fact that `subst` is folded first. */
    pub constraints: Vec<InEnvironment<Constraint<I>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Fold, HasInterner)]
pub struct AnswerSubst<I: Interner> {
    pub subst: Substitution<I>, /* NB: The `is_trivial` routine relies on the fact that `subst` is folded first. */
    pub constraints: Vec<InEnvironment<Constraint<I>>>,
    pub delayed_subgoals: Vec<InEnvironment<Goal<I>>>,
}
