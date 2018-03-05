use chalk_parse::ast;
use fallible::*;
use fold::{DefaultTypeFolder, ExistentialFolder, Fold, IdentityUniversalFolder};
use fold::shift::Shift;
use lalrpop_intern::InternedString;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[macro_use]
mod macros;

crate mod could_match;

crate type Identifier = InternedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// From type-name to item-id. Used during lowering only.
    crate type_ids: BTreeMap<Identifier, ItemId>,

    /// For each struct/trait:
    crate type_kinds: BTreeMap<ItemId, TypeKind>,

    /// For each struct:
    crate struct_data: BTreeMap<ItemId, StructDatum>,

    /// For each impl:
    crate impl_data: BTreeMap<ItemId, ImplDatum>,

    /// For each trait:
    crate trait_data: BTreeMap<ItemId, TraitDatum>,

    /// For each associated ty:
    crate associated_ty_data: BTreeMap<ItemId, AssociatedTyDatum>,

    /// For each default impl (automatically generated for auto traits):
    crate default_impl_data: Vec<DefaultImplDatum>,

    /// For each user-specified clause
    crate custom_clauses: Vec<ProgramClause<DomainGoal>>,
}

impl Program {
    /// Used for debugging output
    crate fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy,
    ) -> (&AssociatedTyDatum, &'p [Parameter], &'p [Parameter]) {
        let ProjectionTy {
            associated_ty_id,
            ref parameters,
        } = *projection;
        let associated_ty_data = &self.associated_ty_data[&associated_ty_id];
        let trait_datum = &self.trait_data[&associated_ty_data.trait_id];
        let trait_num_params = trait_datum.binders.len();
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data, trait_params, other_params)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment<D> {
    /// For each trait (used for debugging):
    crate trait_data: BTreeMap<ItemId, TraitDatum>,

    /// For each associated type (used for debugging):
    crate associated_ty_data: BTreeMap<ItemId, AssociatedTyDatum>,

    /// Compiled forms of the above:
    crate program_clauses: Vec<ProgramClause<D>>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The set of assumptions we've made so far, and the current number of
/// universal (forall) quantifiers we're within.
pub struct Environment<D> {
    crate clauses: Vec<D>,
}

impl<D: Clone + Ord> Environment<D> {
    crate fn new() -> Arc<Self> {
        Arc::new(Environment { clauses: vec![] })
    }

    crate fn add_clauses<I>(&self, clauses: I) -> Arc<Self>
    where
        I: IntoIterator<Item = D>,
    {
        let mut env = self.clone();
        let env_clauses: BTreeSet<_> = env.clauses.into_iter().chain(clauses).collect();
        env.clauses = env_clauses.into_iter().collect();
        Arc::new(env)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InEnvironment<G: EnvironmentArg> {
    crate environment: Arc<Environment<G::DomainGoal>>,
    crate goal: G,
}

pub trait EnvironmentArg: Sized + Fold<Result = Self> {
    type DomainGoal: Fold<Result = Self::DomainGoal>;
}

impl<G: EnvironmentArg> InEnvironment<G> {
    crate fn new(environment: &Arc<Environment<G::DomainGoal>>, goal: G) -> Self {
        InEnvironment {
            environment: environment.clone(),
            goal,
        }
    }

    crate fn map<OP, H>(self, op: OP) -> InEnvironment<H>
    where
        OP: FnOnce(G) -> H,
        H: EnvironmentArg<DomainGoal = G::DomainGoal>,
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
    crate fn to_ty(self) -> Ty {
        Ty::Apply(ApplicationTy {
            name: self,
            parameters: vec![],
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseIndex {
    crate counter: usize,
}

impl UniverseIndex {
    crate const ROOT: UniverseIndex = UniverseIndex { counter: 0 };

    crate fn root() -> UniverseIndex {
        Self::ROOT
    }

    crate fn can_see(self, ui: UniverseIndex) -> bool {
        self.counter >= ui.counter
    }

    crate fn to_lifetime(self) -> Lifetime {
        Lifetime::ForAll(self)
    }

    crate fn next(self) -> UniverseIndex {
        UniverseIndex {
            counter: self.counter + 1,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    crate index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeKind {
    crate sort: TypeSort,
    crate name: Identifier,
    crate binders: Binders<()>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatum {
    crate binders: Binders<ImplDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatumBound {
    crate trait_ref: PolarizedTraitRef,
    crate where_clauses: Vec<DomainGoal>,
    crate associated_ty_values: Vec<AssociatedTyValue>,
    crate specialization_priority: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatum {
    crate binders: Binders<DefaultImplDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatumBound {
    crate trait_ref: TraitRef,
    crate accessible_tys: Vec<Ty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum {
    crate binders: Binders<StructDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatumBound {
    crate self_ty: ApplicationTy,
    crate fields: Vec<Ty>,
    crate where_clauses: Vec<DomainGoal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatum {
    crate binders: Binders<TraitDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatumBound {
    crate trait_ref: TraitRef,
    crate where_clauses: Vec<DomainGoal>,
    crate flags: TraitFlags,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitFlags {
    crate auto: bool,
    crate marker: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyDatum {
    /// The trait this associated type is defined in.
    crate trait_id: ItemId,

    /// The ID of this associated type
    crate id: ItemId,

    /// Name of this associated type.
    crate name: Identifier,

    /// Parameters on this associated type, beginning with those from the trait,
    /// but possibly including more.
    crate parameter_kinds: Vec<ParameterKind<Identifier>>,

    /// Where clauses that must hold for the projection be well-formed.
    crate where_clauses: Vec<DomainGoal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyValue {
    crate associated_ty_id: ItemId,

    // note: these binders are in addition to those from the impl
    crate value: Binders<AssociatedTyValueBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyValueBound {
    /// Type that we normalize to. The X in `type Foo<'a> = X`.
    crate ty: Ty,

    /// Where-clauses that must hold for projection to be valid. The
    /// WC in `type Foo<'a> = X where WC`.
    crate where_clauses: Vec<DomainGoal>,
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
    crate fn as_projection_ty_enum(&self) -> ProjectionTyRefEnum {
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
    crate num_binders: usize,
    crate ty: Ty,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lifetime {
    /// See Ty::Var(_).
    Var(usize),
    ForAll(UniverseIndex),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ApplicationTy {
    crate name: TypeName,
    crate parameters: Vec<Parameter>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParameterKind<T, L = T> {
    Ty(T),
    Lifetime(L),
}

impl<T> ParameterKind<T> {
    crate fn into_inner(self) -> T {
        match self {
            ParameterKind::Ty(t) => t,
            ParameterKind::Lifetime(t) => t,
        }
    }

    crate fn map<OP, U>(self, op: OP) -> ParameterKind<U>
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
    crate fn assert_ty_ref(&self) -> &T {
        self.as_ref().ty().unwrap()
    }

    crate fn assert_lifetime_ref(&self) -> &L {
        self.as_ref().lifetime().unwrap()
    }

    crate fn as_ref(&self) -> ParameterKind<&T, &L> {
        match *self {
            ParameterKind::Ty(ref t) => ParameterKind::Ty(t),
            ParameterKind::Lifetime(ref l) => ParameterKind::Lifetime(l),
        }
    }

    crate fn ty(self) -> Option<T> {
        match self {
            ParameterKind::Ty(t) => Some(t),
            _ => None,
        }
    }

    crate fn lifetime(self) -> Option<L> {
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

crate type Parameter = ParameterKind<Ty, Lifetime>;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectionTy {
    crate associated_ty_id: ItemId,
    crate parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnselectedProjectionTy {
    crate type_name: Identifier,
    crate parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectionTyEnum<S = ProjectionTy, U = UnselectedProjectionTy> {
    Selected(S),
    Unselected(U),
}

crate type ProjectionTyRefEnum<'a> = ProjectionTyEnum<&'a ProjectionTy, &'a UnselectedProjectionTy>;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TraitRef {
    crate trait_id: ItemId,
    crate parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum PolarizedTraitRef {
    Positive(TraitRef),
    Negative(TraitRef),
}

impl PolarizedTraitRef {
    crate fn is_positive(&self) -> bool {
        match *self {
            PolarizedTraitRef::Positive(_) => true,
            PolarizedTraitRef::Negative(_) => false,
        }
    }

    crate fn trait_ref(&self) -> &TraitRef {
        match *self {
            PolarizedTraitRef::Positive(ref tr) | PolarizedTraitRef::Negative(ref tr) => tr,
        }
    }
}

/// A "domain goal" is a goal that is directly about Rust, rather than a pure
/// logical statement. As much as possible, the Chalk solver should avoid
/// decomposing this enum, and instead treat its values opaquely.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DomainGoal {
    Implemented(TraitRef),
    ProjectionEq(ProjectionEq),
    Normalize(Normalize),
    UnselectedNormalize(UnselectedNormalize),
    WellFormed(WellFormed),
    FromEnv(FromEnv),
    InScope(ItemId),
}

impl DomainGoal {
    /// Lift a goal to a corresponding program clause (with a trivial
    /// antecedent).
    crate fn into_program_clause(self) -> ProgramClause<DomainGoal> {
        ProgramClause {
            implication: Binders {
                value: ProgramClauseImplication {
                    consequence: self,
                    conditions: vec![],
                },
                binders: vec![],
            },
        }
    }

    /// Turn a where clause into the WF version of it i.e.:
    /// * `T: Trait` maps to `WellFormed(T: Trait)`
    /// * `T: Trait<Item = Foo>` maps to `WellFormed(T: Trait<Item = Foo>)`
    /// * any other clause maps to itself
    crate fn into_well_formed_clause(self) -> DomainGoal {
        match self {
            DomainGoal::Implemented(tr) => DomainGoal::WellFormed(WellFormed::TraitRef(tr)),
            DomainGoal::ProjectionEq(n) => DomainGoal::WellFormed(WellFormed::ProjectionEq(n)),
            goal => goal,
        }
    }

    /// Same as `into_well_formed_clause` but with the `FromEnv` predicate instead of `WellFormed`.
    crate fn into_from_env_clause(self) -> DomainGoal {
        match self {
            DomainGoal::Implemented(tr) => DomainGoal::FromEnv(FromEnv::TraitRef(tr)),
            DomainGoal::ProjectionEq(n) => DomainGoal::FromEnv(FromEnv::ProjectionEq(n)),
            goal => goal,
        }
    }
}

impl EnvironmentArg for DomainGoal {
    type DomainGoal = Self;
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A goal that does not involve any logical connectives. Equality is treated
/// specially by the logic (as with most first-order logics), since it interacts
/// with unification etc.
pub enum LeafGoal<D> {
    EqGoal(EqGoal),
    DomainGoal(D),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EqGoal {
    crate a: Parameter,
    crate b: Parameter,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A predicate which is true is some object is well-formed, e.g. a type or a trait ref.
/// For example, given the following type definition:
///
/// ```notrust
/// struct Set<K> where K: Hash {
///     ...
/// }
/// ```
///
/// then we have the following rule: `WellFormed(Set<K>) :- (K: Hash)`.
/// See the complete rules in `lower.rs`.
pub enum WellFormed {
    Ty(Ty),
    TraitRef(TraitRef),
    ProjectionEq(ProjectionEq),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A predicate which enables deriving everything which should be true if we *know* that some object
/// is well-formed. For example, given the following trait definitions:
///
/// ```notrust
/// trait Clone { ... }
/// trait Copy where Self: Clone { ... }
/// ```
///
/// then we can use `FromEnv(T: Copy)` to derive that `T: Clone`, like in:
///
/// ```notrust
/// forall<T> {
///     if (FromEnv(T: Copy)) {
///         T: Clone
///     }
/// }
/// ```
///
/// See the complete rules in `lower.rs`.
pub enum FromEnv {
    Ty(Ty),
    TraitRef(TraitRef),
    ProjectionEq(ProjectionEq),
}

/// Proves that the given projection **normalizes** to the given
/// type. A projection `T::Foo` normalizes to the type `U` if we can
/// **match it to an impl** and that impl has a `type Foo = V` where
/// `U = V`.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Normalize {
    crate projection: ProjectionTy,
    crate ty: Ty,
}

/// Proves **equality** between a projection `T::Foo` and a type
/// `U`. Equality can be proven via normalization, but we can also
/// prove that `T::Foo = V::Foo` if `T = V` without normalizing.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectionEq {
    crate projection: ProjectionTy,
    crate ty: Ty,
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
    crate projection: UnselectedProjectionTy,
    crate ty: Ty,
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
    crate binders: Vec<ParameterKind<()>>,
    crate value: T,
}

impl<T> Binders<T> {
    crate fn map_ref<U, OP>(&self, op: OP) -> Binders<U>
    where
        OP: FnOnce(&T) -> U,
    {
        let value = op(&self.value);
        Binders {
            binders: self.binders.clone(),
            value: value,
        }
    }

    crate fn len(&self) -> usize {
        self.binders.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramClause<D> {
    crate implication: Binders<ProgramClauseImplication<D>>,
}

/// Represents one clause of the form `consequence :- conditions` where
/// `conditions = cond_1 && cond_2 && ...` is the conjunction of the individual
/// conditions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramClauseImplication<D> {
    crate consequence: D,
    crate conditions: Vec<Goal<D>>,
}

/// Wraps a "canonicalized item". Items are canonicalized as follows:
///
/// All unresolved existential variables are "renumbered" according to their
/// first appearance; the kind/universe of the variable is recorded in the
/// `binders` field.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Canonical<T> {
    crate value: T,
    crate binders: Vec<ParameterKind<UniverseIndex>>,
}

impl<T> Canonical<T> {
    /// Maps the contents using `op`, but preserving the binders.
    ///
    /// NB. `op` will be invoked with an instantiated version of the
    /// canonical value, where inference variables (from a fresh
    /// inference context) are used in place of the quantified free
    /// variables. The result should be in terms of those same
    /// inference variables and will be re-canonicalized.
    crate fn map<OP, U>(self, op: OP) -> Canonical<U::Result>
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
        use solve::infer::InferenceTable;
        let mut infer = InferenceTable::new();
        let snapshot = infer.snapshot();
        let instantiated_value = infer.instantiate_canonical(&self);
        let mapped_value = op(instantiated_value);
        let result = infer.canonicalize(&mapped_value);
        infer.rollback_to(snapshot);
        result.quantified
    }

    /// Substitutes the values from `subst` in place of the values
    /// bound by the binders in this canonical; the substitution should be
    /// complete.
    crate fn substitute(&self, mut subst: &Substitution) -> T::Result
    where
        T: Fold,
    {
        assert_eq!(
            subst.parameters.len(),
            self.binders.len(),
            "substitute invoked with incomplete substitution",
        );
        self.value.fold_with(&mut subst, 0).unwrap()
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
    crate canonical: Canonical<T>,
    crate universes: usize,
}

impl<T> UCanonical<T> {
    crate fn trivial_substitution(&self) -> Substitution {
        let binders = &self.canonical.binders;
        Substitution {
            parameters: binders
                .iter()
                .enumerate()
                .map(|(index, pk)| match pk {
                    ParameterKind::Ty(_) => ParameterKind::Ty(Ty::Var(index)),
                    ParameterKind::Lifetime(_) => ParameterKind::Lifetime(Lifetime::Var(index)),
                })
                .collect(),
        }
    }

    crate fn is_trivial_substitution(&self, canonical_subst: &Canonical<ConstrainedSubst>) -> bool {
        let subst = &canonical_subst.value.subst;
        assert_eq!(self.canonical.binders.len(), subst.parameters.len());
        // A subst is trivial if..
        subst.parameters.iter().zip(0..).all(|(parameter, index)| {
            // All types and lifetimes are mapped to distinct
            // variables.  Since this has been canonicalized, and
            // the substitution appears first, those will also be
            // the first N variables.
            match parameter {
                ParameterKind::Ty(Ty::Var(depth)) => index == *depth,
                ParameterKind::Lifetime(Lifetime::Var(depth)) => index == *depth,
                _ => false,
            }
        })
    }
}

impl UCanonical<InEnvironment<Goal<DomainGoal>>> {
    /// A goal has coinductive semantics if it is of the form `T: AutoTrait`, or if it is of the
    /// form `WellFormed(T: Trait)` where `Trait` is any trait. The latter is needed for dealing
    /// with WF requirements and cyclic traits, which generates cycles in the proof tree which must
    /// not be rejected but instead must be treated as a success.
    crate fn is_coinductive(&self, program: &ProgramEnvironment<DomainGoal>) -> bool {
        match &self.canonical.value.goal {
            Goal::Leaf(LeafGoal::DomainGoal(DomainGoal::Implemented(tr))) => {
                let trait_datum = &program.trait_data[&tr.trait_id];
                trait_datum.binders.value.flags.auto
            }
            Goal::Leaf(LeafGoal::DomainGoal(DomainGoal::WellFormed(WellFormed::TraitRef(_)))) => {
                true
            }
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum Goal<D> {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    Quantified(QuantifierKind, Binders<Box<Goal<D>>>),
    Implies(Vec<D>, Box<Goal<D>>),
    And(Box<Goal<D>>, Box<Goal<D>>),
    Not(Box<Goal<D>>),
    Leaf(LeafGoal<D>),

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

impl Goal<DomainGoal> {
    crate fn quantify(
        self,
        kind: QuantifierKind,
        binders: Vec<ParameterKind<()>>,
    ) -> Goal<DomainGoal> {
        Goal::Quantified(
            kind,
            Binders {
                value: Box::new(self),
                binders,
            },
        )
    }

    crate fn implied_by(self, predicates: Vec<DomainGoal>) -> Goal<DomainGoal> {
        Goal::Implies(predicates, Box::new(self))
    }

    /// Returns a canonical goal in which the outermost `exists<>` and
    /// `forall<>` quantifiers (as well as implications) have been
    /// "peeled" and are converted into free universal or existential
    /// variables. Assumes that this goal is a "closed goal" which
    /// does not -- at present -- contain any variables. Useful for
    /// REPLs and tests but not much else.
    pub fn into_peeled_goal(self) -> UCanonical<InEnvironment<Goal<DomainGoal>>> {
        use solve::infer::InferenceTable;
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
    crate fn into_closed_goal(self) -> UCanonical<InEnvironment<Goal<DomainGoal>>> {
        use solve::infer::InferenceTable;
        let mut infer = InferenceTable::new();
        let env_goal = InEnvironment::new(&Environment::new(), self);
        let canonical_goal = infer.canonicalize(&env_goal).quantified;
        infer.u_canonicalize(&canonical_goal).quantified
    }
}

impl<D: EnvironmentArg> EnvironmentArg for Goal<D> {
    type DomainGoal = D;
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

impl EnvironmentArg for Constraint {
    type DomainGoal = DomainGoal;
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
    crate parameters: Vec<Parameter>,
}

impl Substitution {
    crate fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }
}

impl<'a> DefaultTypeFolder for &'a Substitution {}

impl<'a> ExistentialFolder for &'a Substitution {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Fallible<Ty> {
        let ty = &self.parameters[depth];
        let ty = ty.assert_ty_ref();
        Ok(ty.up_shift(binders))
    }

    fn fold_free_existential_lifetime(
        &mut self,
        depth: usize,
        binders: usize,
    ) -> Fallible<Lifetime> {
        let l = &self.parameters[depth];
        let l = l.assert_lifetime_ref();
        Ok(l.up_shift(binders))
    }
}

impl<'a> IdentityUniversalFolder for &'a Substitution {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstrainedSubst {
    crate subst: Substitution,
    crate constraints: Vec<InEnvironment<Constraint>>,
}

crate mod debug;
pub mod tls;
