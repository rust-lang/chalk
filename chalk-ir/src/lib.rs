#![feature(non_modrs_mods)]
#![feature(crate_visibility_modifier)]
#![feature(crate_in_paths)]
#![feature(specialization)]

use chalk_engine::fallible::*;
use chalk_parse::ast;
use cast::Cast;
use fold::shift::Shift;
use fold::{DefaultTypeFolder, ExistentialFolder, Fold, IdentityUniversalFolder};
use lalrpop_intern::InternedString;
use std::collections::BTreeSet;
use std::iter;
use std::sync::Arc;

#[macro_use]
extern crate chalk_macros;

extern crate chalk_engine;
extern crate chalk_parse;
extern crate ena;
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
pub mod infer;
pub mod solve;
pub mod tls;

pub type Identifier = InternedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// Indicates whether a given trait has coinductive semantics --
    /// at present, this is true only for auto traits.
    pub coinductive_traits: BTreeSet<ItemId>,

    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause>,
}

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
    ItemId(ItemId),

    /// skolemized form of a type parameter like `T`
    ForAll(UniverseIndex),

    /// an associated type like `Iterator::Item`; see `AssociatedType` for details
    AssociatedType(ItemId),
}

impl TypeName {
    pub fn to_ty(self) -> Ty {
        Ty::Apply(ApplicationTy {
            name: self,
            parameters: vec![],
        })
    }
}

/// An universe index is how a universally quantified parameter is
/// represented when it's binder is moved into the environment.
/// An example chain of transformations would be:
/// `forall<T> { Goal(T) }` (syntatical representation)
/// `forall { Goal(?0) }` (used a DeBruijn index)
/// `Goal(!U1)` (the quantifier was moved to the environment and replaced with a universe index)
/// See https://rust-lang-nursery.github.io/rustc-guide/mir/regionck.html#skol for more.
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

    pub fn to_lifetime(self) -> Lifetime {
        Lifetime::ForAll(self)
    }

    pub fn next(self) -> UniverseIndex {
        UniverseIndex {
            counter: self.counter + 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    pub index: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ty {
    /// References the binding at the given depth (deBruijn index
    /// style). In an inference context (i.e., when solving goals),
    /// free bindings refer into the inference table.
    Var(usize),
    Apply(ApplicationTy),
    Projection(ProjectionTy),
    UnselectedProjection(UnselectedProjectionTy),
    ForAll(Box<QuantifiedTy>),
}

impl Ty {
    pub fn as_projection_ty_enum(&self) -> ProjectionTyRefEnum {
        match *self {
            Ty::Projection(ref proj) => ProjectionTyEnum::Selected(proj),
            Ty::UnselectedProjection(ref proj) => ProjectionTyEnum::Unselected(proj),
            _ => panic!("{:?} is not a projection", self),
        }
    }

    pub fn is_projection(&self) -> bool {
        match *self {
            Ty::Projection(..) | Ty::UnselectedProjection(..) => true,
            _ => false,
        }
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
    Var(usize),
    ForAll(UniverseIndex),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ApplicationTy {
    pub name: TypeName,
    pub parameters: Vec<Parameter>,
}

impl ApplicationTy {
    pub fn type_parameters<'a>(&'a self) -> impl Iterator<Item = Ty> + 'a {
        // This unwrap() is safe because is_ty ensures that we definitely have a Ty
        self.parameters
            .iter()
            .filter(|p| p.is_ty())
            .map(|p| p.clone().ty().unwrap())
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

impl<T, L> ast::Kinded for ParameterKind<T, L> {
    fn kind(&self) -> ast::Kind {
        match *self {
            ParameterKind::Ty(_) => ast::Kind::Ty,
            ParameterKind::Lifetime(_) => ast::Kind::Lifetime,
        }
    }
}

pub type Parameter = ParameterKind<Ty, Lifetime>;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectionTy {
    pub associated_ty_id: ItemId,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnselectedProjectionTy {
    pub type_name: Identifier,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectionTyEnum<S = ProjectionTy, U = UnselectedProjectionTy> {
    Selected(S),
    Unselected(U),
}

pub type ProjectionTyRefEnum<'a> = ProjectionTyEnum<&'a ProjectionTy, &'a UnselectedProjectionTy>;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitRef {
    pub trait_id: ItemId,
    pub parameters: Vec<Parameter>,
}

impl TraitRef {
    pub fn type_parameters<'a>(&'a self) -> impl Iterator<Item = Ty> + 'a {
        // This unwrap() is safe because is_ty ensures that we definitely have a Ty
        self.parameters
            .iter()
            .filter(|p| p.is_ty())
            .map(|p| p.clone().ty().unwrap())
    }
}

/// Where clauses that can be written by a Rust programmer.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WhereClause {
    Implemented(TraitRef),
    ProjectionEq(ProjectionEq),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Derefs {
    pub source: Ty,
    pub target: Ty,
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
    UnselectedNormalize(UnselectedNormalize),

    InScope(ItemId),

    /// Whether a type can deref into another. Right now this is just:
    /// ```notrust
    /// Derefs(T, U) :- Implemented(T: Deref<Target = U>)
    /// ```
    /// In Rust there are also raw pointers which can be deref'd but do not implement Deref.
    Derefs(Derefs),

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

/// Indicates that the trait where the associated type belongs to is
/// not yet known, i.e. is unselected. For example, a normal
/// `Normalize` would be of the form `<Vec<T> as Iterator>::Item ->
/// T`. When `Iterator` is in scope, and it is the only trait in scope
/// with an associated type `Item`, it suffices to write
/// `Vec<T>::Item` instead of `<Vec<T> as Iterator>::Item`. The
/// corresponding `UnselectedNormalize` is `Vec<T>::Item -> T`.
///
/// For each associated type we encounter in an `impl`, we generate
/// rules to derive an `UnselectedNormalize` from a `Normalize`. For
/// example, implementing `Iterator` for `Vec<T>` yields the rule:
///
/// ```text
/// Vec<T>::Item -> T :-
///     InScope(Iterator),
///     <Vec<T> as Iterator>::Item -> T
/// ```
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnselectedNormalize {
    pub projection: UnselectedProjectionTy,
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
        let new_var = Ty::Var(0);
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProgramClause {
    Implies(ProgramClauseImplication),
    ForAll(Binders<ProgramClauseImplication>),
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

impl<T> Canonical<T> {
    /// Maps the contents using `op`, but preserving the binders.
    ///
    /// NB. `op` will be invoked with an instantiated version of the
    /// canonical value, where inference variables (from a fresh
    /// inference context) are used in place of the quantified free
    /// variables. The result should be in terms of those same
    /// inference variables and will be re-canonicalized.
    pub fn map<OP, U>(self, op: OP) -> Canonical<U::Result>
    where
        OP: FnOnce(T::Result) -> U,
        T: Fold,
        U: Fold,
    {
        // Subtle: It is only quite rarely correct to apply `op` and
        // just re-use our existing binders. For that to be valid, the
        // result of `op` would have to ensure that it re-uses all the
        // existing free variables and in the same order. Otherwise,
        // the canonical form would be different: the variables might
        // be numbered differently, or some may not longer be used.
        // This would mean that two canonical values could no longer
        // be compared with `Eq`, which defeats a key invariant of the
        // `Canonical` type (indeed, its entire reason for existence).
        use self::infer::InferenceTable;
        let mut infer = InferenceTable::new();
        let snapshot = infer.snapshot();
        let instantiated_value = infer.instantiate_canonical(&self);
        let mapped_value = op(instantiated_value);
        let result = infer.canonicalize(&mapped_value);
        infer.rollback_to(snapshot);
        result.quantified
    }
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

impl UCanonical<InEnvironment<Goal>> {
    /// A goal has coinductive semantics if it is of the form `T: AutoTrait`, or if it is of the
    /// form `WellFormed(T: Trait)` where `Trait` is any trait. The latter is needed for dealing
    /// with WF requirements and cyclic traits, which generates cycles in the proof tree which must
    /// not be rejected but instead must be treated as a success.
    pub fn is_coinductive(&self, program: &ProgramEnvironment) -> bool {
        self.canonical.value.goal.is_coinductive(program)
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
            }.with_fresh_type_var(|goal, ty| {
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

    /// Returns a canonical goal in which the outermost `exists<>` and
    /// `forall<>` quantifiers (as well as implications) have been
    /// "peeled" and are converted into free universal or existential
    /// variables. Assumes that this goal is a "closed goal" which
    /// does not -- at present -- contain any variables. Useful for
    /// REPLs and tests but not much else.
    pub fn into_peeled_goal(self) -> UCanonical<InEnvironment<Goal>> {
        use self::infer::InferenceTable;
        let mut infer = InferenceTable::new();
        let peeled_goal = {
            let mut env_goal = InEnvironment::new(&Environment::new(), self);
            loop {
                let InEnvironment { environment, goal } = env_goal;
                match goal {
                    Goal::Quantified(QuantifierKind::ForAll, subgoal) => {
                        let subgoal = infer.instantiate_binders_universally(&subgoal);
                        env_goal = InEnvironment::new(&environment, *subgoal);
                    }

                    Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                        let subgoal = infer.instantiate_binders_existentially(&subgoal);
                        env_goal = InEnvironment::new(&environment, *subgoal);
                    }

                    Goal::Implies(wc, subgoal) => {
                        let new_environment = &environment.add_clauses(wc);
                        env_goal = InEnvironment::new(&new_environment, *subgoal);
                    }

                    _ => break InEnvironment::new(&environment, goal),
                }
            }
        };
        let canonical = infer.canonicalize(&peeled_goal).quantified;
        infer.u_canonicalize(&canonical).quantified
    }

    /// Given a goal with no free variables (a "closed" goal), creates
    /// a canonical form suitable for solving. This is a suitable
    /// choice if you don't actually care about the values of any of
    /// the variables within; otherwise, you might want
    /// `into_peeled_goal`.
    ///
    /// # Panics
    ///
    /// Will panic if this goal does in fact contain free variables.
    pub fn into_closed_goal(self) -> UCanonical<InEnvironment<Goal>> {
        use self::infer::InferenceTable;
        let mut infer = InferenceTable::new();
        let env_goal = InEnvironment::new(&Environment::new(), self);
        let canonical_goal = infer.canonicalize(&env_goal).quantified;
        infer.u_canonicalize(&canonical_goal).quantified
    }

    pub fn is_coinductive(&self, program: &ProgramEnvironment) -> bool {
        match self {
            Goal::Leaf(LeafGoal::DomainGoal(DomainGoal::Holds(wca))) => match wca {
                WhereClause::Implemented(tr) => program.coinductive_traits.contains(&tr.trait_id),
                WhereClause::ProjectionEq(..) => false,
            },
            Goal::Leaf(LeafGoal::DomainGoal(DomainGoal::WellFormed(WellFormed::Trait(..)))) => true,
            Goal::Quantified(QuantifierKind::ForAll, goal) => goal.value.is_coinductive(program),
            _ => false,
        }
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
    ///
    /// This is a map because the substitution is not necessarily
    /// complete. We use a btree map to ensure that the result is in a
    /// deterministic order.
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
            .all(|(parameter, index)| match parameter {
                ParameterKind::Ty(Ty::Var(depth)) => index == *depth,
                ParameterKind::Lifetime(Lifetime::Var(depth)) => index == *depth,
                _ => false,
            })
    }
}

impl<'a> DefaultTypeFolder for &'a Substitution {}

impl<'a> ExistentialFolder for &'a Substitution {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        let ty = &self.parameters[depth];
        let ty = ty.assert_ty_ref();
        Ok(ty.shifted_in(binders))
    }

    fn fold_free_existential_lifetime(
        &mut self,
        depth: usize,
        binders: usize,
    ) -> Fallible<Lifetime> {
        let l = &self.parameters[depth];
        let l = l.assert_lifetime_ref();
        Ok(l.shifted_in(binders))
    }
}

impl<'a> IdentityUniversalFolder for &'a Substitution {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstrainedSubst {
    pub subst: Substitution,
    pub constraints: Vec<InEnvironment<Constraint>>,
}
