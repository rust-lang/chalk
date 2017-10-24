use cast::Cast;
use chalk_parse::ast;
use fold::{DefaultTypeFolder, Fold, ExistentialFolder, IdentityUniversalFolder};
use lalrpop_intern::InternedString;
use solve::infer::InferenceVariable;
use std::collections::{HashSet, HashMap, BTreeMap};
use std::sync::Arc;

#[macro_use] mod macros;

pub mod could_match;

pub type Identifier = InternedString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    /// From type-name to item-id. Used during lowering only.
    pub type_ids: HashMap<Identifier, ItemId>,

    /// For each struct/trait:
    pub type_kinds: HashMap<ItemId, TypeKind>,

    /// For each struct:
    pub struct_data: HashMap<ItemId, StructDatum>,

    /// For each impl:
    pub impl_data: HashMap<ItemId, ImplDatum>,

    /// For each trait:
    pub trait_data: HashMap<ItemId, TraitDatum>,

    /// For each associated ty:
    pub associated_ty_data: HashMap<ItemId, AssociatedTyDatum>,

    /// For each default impl (automatically generated for auto traits):
    pub default_impl_data: Vec<DefaultImplDatum>,

    /// For each user-specified clause
    pub custom_clauses: Vec<ProgramClause>,
}

impl Program {
    /// Used for debugging output
    pub fn split_projection<'p>(&self, projection: &'p ProjectionTy)
                            -> (&AssociatedTyDatum, &'p [Parameter], &'p [Parameter]) {
        let ProjectionTy { associated_ty_id, ref parameters } = *projection;
        let associated_ty_data = &self.associated_ty_data[&associated_ty_id];
        let trait_datum = &self.trait_data[&associated_ty_data.trait_id];
        let trait_num_params = trait_datum.binders.len();
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data, trait_params, other_params)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramEnvironment {
    /// For each trait (used for debugging):
    pub trait_data: HashMap<ItemId, TraitDatum>,

    /// For each associated type (used for debugging):
    pub associated_ty_data: HashMap<ItemId, AssociatedTyDatum>,

    /// Compiled forms of the above:
    pub program_clauses: Vec<ProgramClause>,
}

impl ProgramEnvironment {
    /// Used for debugging output
    pub fn split_projection<'p>(&self, projection: &'p ProjectionTy)
                            -> (&AssociatedTyDatum, &'p [Parameter], &'p [Parameter]) {
        let ProjectionTy { associated_ty_id, ref parameters } = *projection;
        let associated_ty_data = &self.associated_ty_data[&associated_ty_id];
        let trait_datum = &self.trait_data[&associated_ty_data.trait_id];
        let trait_num_params = trait_datum.binders.len();
        let split_point = parameters.len() - trait_num_params;
        let (other_params, trait_params) = parameters.split_at(split_point);
        (associated_ty_data, trait_params, other_params)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The set of assumptions we've made so far, and the current number of
/// universal (forall) quantifiers we're within.
pub struct Environment {
    pub universe: UniverseIndex,
    pub clauses: Vec<DomainGoal>,
}

impl Environment {
    pub fn new() -> Arc<Environment> {
        Arc::new(Environment { universe: UniverseIndex::root(), clauses: vec![] })
    }

    pub fn add_clauses<I>(&self, clauses: I) -> Arc<Environment>
        where I: IntoIterator<Item = DomainGoal>
    {
        let mut env = self.clone();
        let env_clauses: HashSet<_> = env.clauses.into_iter().chain(clauses).collect();
        env.clauses = env_clauses.into_iter().collect();
        Arc::new(env)
    }

    pub fn new_universe(&self) -> Arc<Environment> {
        let mut env = self.clone();
        env.universe = UniverseIndex { counter: self.universe.counter + 1 };
        Arc::new(env)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InEnvironment<G> {
    pub environment: Arc<Environment>,
    pub goal: G,
}

impl<G> InEnvironment<G> {
    pub fn new(environment: &Arc<Environment>, goal: G) -> Self {
        InEnvironment { environment: environment.clone(), goal }
    }

    pub fn empty(goal: G) -> Self {
        InEnvironment { environment: Environment::new(), goal }
    }

    pub fn map<OP, H>(self, op: OP) -> InEnvironment<H>
        where OP: FnOnce(G) -> H
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
    pub fn is_for_all(&self) -> bool {
        match *self {
            TypeName::ForAll(_) => true,
            _ => false,
        }
    }

    pub fn to_ty(self) -> Ty {
        Ty::Apply(ApplicationTy { name: self, parameters: vec![] })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseIndex {
    pub counter: usize,
}

impl UniverseIndex {
    pub fn root() -> UniverseIndex {
        UniverseIndex { counter: 0 }
    }

    pub fn can_see(self, ui: UniverseIndex) -> bool {
        self.counter >= ui.counter
    }

    pub fn is_root(self) -> bool {
        self.counter == 0
    }

    pub fn to_lifetime(self) -> Lifetime {
        Lifetime::ForAll(self)
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
    pub binders: Binders<()>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeSort {
    Struct,
    Trait,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatum {
    pub binders: Binders<ImplDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImplDatumBound {
    pub trait_ref: PolarizedTraitRef,
    pub where_clauses: Vec<DomainGoal>,
    pub associated_ty_values: Vec<AssociatedTyValue>,
    pub specialization_priority: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatum {
    pub binders: Binders<DefaultImplDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DefaultImplDatumBound {
    pub trait_ref: TraitRef,
    pub accessible_tys: Vec<Ty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum {
    pub binders: Binders<StructDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatumBound {
    pub self_ty: ApplicationTy,
    pub fields: Vec<Ty>,
    pub where_clauses: Vec<DomainGoal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatum {
    pub binders: Binders<TraitDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitDatumBound {
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<DomainGoal>,
    pub flags: TraitFlags,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitFlags {
    pub auto: bool,
    pub marker: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyDatum {
    /// The trait this associated type is defined in.
    pub trait_id: ItemId,

    /// The ID of this associated type
    pub id: ItemId,

    /// Name of this associated type.
    pub name: Identifier,

    /// Parameters on this associated type, beginning with those from the trait,
    /// but possibly including more.
    pub parameter_kinds: Vec<ParameterKind<Identifier>>,

    /// Where clauses that must hold for the projection be well-formed.
    pub where_clauses: Vec<DomainGoal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyValue {
    pub associated_ty_id: ItemId,

    // note: these binders are in addition to those from the impl
    pub value: Binders<AssociatedTyValueBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssociatedTyValueBound {
    /// Type that we normalize to. The X in `type Foo<'a> = X`.
    pub ty: Ty,

    /// Where-clauses that must hold for projection to be valid. The
    /// WC in `type Foo<'a> = X where WC`.
    pub where_clauses: Vec<DomainGoal>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ApplicationTy {
    pub name: TypeName,
    pub parameters: Vec<Parameter>,
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
pub struct TraitRef {
    pub trait_id: ItemId,
    pub parameters: Vec<Parameter>,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum PolarizedTraitRef {
    Positive(TraitRef),
    Negative(TraitRef),
}

impl PolarizedTraitRef {
    pub fn is_positive(&self) -> bool {
        match *self {
            PolarizedTraitRef::Positive(_) => true,
            PolarizedTraitRef::Negative(_) => false,
        }
    }

    pub fn trait_ref(&self) -> &TraitRef {
        match *self {
            PolarizedTraitRef::Positive(ref tr) |
            PolarizedTraitRef::Negative(ref tr) => tr
        }
    }
}

/// A "domain goal" is a goal that is directly about Rust, rather than a pure
/// logical statement. As much as possible, the Chalk solver should avoid
/// decomposing this enum, and instead treat its values opaquely.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DomainGoal {
    Implemented(TraitRef),
    Normalize(Normalize),
    WellFormed(WellFormed),
}

impl DomainGoal {
    /// Lift a goal to a corresponding program clause (with a trivial
    /// antecedent).
    pub fn into_program_clause(self) -> ProgramClause {
        ProgramClause {
            implication: Binders {
                value: ProgramClauseImplication {
                    consequence: self,
                    conditions: vec![],
                },
                binders: vec![],
            },
            fallback_clause: false,
        }
    }

    /// A clause of the form (T: Foo) expands to (T: Foo), WF(T: Foo).
    /// A clause of the form (T: Foo<Item = U>) expands to (T: Foo<Item = U>), WF(T: Foo).
    pub fn expanded(self, program: &Program) -> impl Iterator<Item = DomainGoal> {
        let mut expanded = vec![];
        match self {
            DomainGoal::Implemented(ref trait_ref) =>
                expanded.push(WellFormed::TraitRef(trait_ref.clone()).cast()),
            DomainGoal::Normalize(Normalize { ref projection, .. }) => {
                let (associated_ty_data, trait_params, _) = program.split_projection(&projection);
                let trait_ref = TraitRef {
                    trait_id: associated_ty_data.trait_id,
                    parameters: trait_params.to_owned()
                };
                expanded.push(WellFormed::TraitRef(trait_ref).cast());
            }
            _ => ()
        };
        expanded.push(self.cast());
        expanded.into_iter()
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

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WellFormed {
    Ty(Ty),
    TraitRef(TraitRef),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Normalize {
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
    pub fn map_ref<U, OP>(&self, op: OP) -> Binders<U>
        where OP: FnOnce(&T) -> U
    {
        let value = op(&self.value);
        Binders {
            binders: self.binders.clone(),
            value: value,
        }
    }

    pub fn len(&self) -> usize {
        self.binders.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramClause {
    pub implication: Binders<ProgramClauseImplication>,

    /// Is this a fallback clause which should get lower priority?
    pub fallback_clause: bool,
}

/// Represents one clause of the form `consequence :- conditions` where
/// `conditions = cond_1 && cond_2 && ...` is the conjunction of the individual
/// conditions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProgramClauseImplication {
    pub consequence: DomainGoal,
    pub conditions: Vec<Goal>,
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

impl Canonical<InEnvironment<LeafGoal>> {
    pub fn into_reduced_goal(self) -> FullyReducedGoal {
        let Canonical { value: InEnvironment { goal, environment }, binders } = self;
        match goal {
            LeafGoal::EqGoal(goal) => {
                let canonical = Canonical { value: InEnvironment { goal, environment }, binders };
                FullyReducedGoal::EqGoal(canonical)
            }
            LeafGoal::DomainGoal(goal) => {
                let canonical = Canonical { value: InEnvironment { goal, environment }, binders };
                FullyReducedGoal::DomainGoal(canonical)
            }
        }
    }
}

/// A goal that has been fully broken down into leaf form, and canonicalized
/// with an environment. These are the goals we do "real work" on.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FullyReducedGoal {
    EqGoal(Canonical<InEnvironment<EqGoal>>),
    DomainGoal(Canonical<InEnvironment<DomainGoal>>),
}

impl FullyReducedGoal {
    pub fn into_binders(self) -> Vec<ParameterKind<UniverseIndex>> {
        match self {
            FullyReducedGoal::EqGoal(Canonical { binders, .. }) |
            FullyReducedGoal::DomainGoal(Canonical { binders, ..}) => binders,
        }
    }

    /// A goal has coinductive semantics if it is of the form `T: AutoTrait`.
    pub fn is_coinductive(&self, program: &ProgramEnvironment) -> bool {
        if let FullyReducedGoal::DomainGoal(Canonical {
                value: InEnvironment {
                    goal: DomainGoal::Implemented(ref tr),
                    ..
                },
                ..
        }) = *self {
            let trait_datum = &program.trait_data[&tr.trait_id];
            return trait_datum.binders.value.flags.auto;
        }

        false
    }
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
        where OP: FnOnce(T::Result) -> U,
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

    pub fn instantiate_with_subst(&self, mut subst: &Substitution) -> T::Result
        where T: Fold
    {
        self.value.fold_with(&mut subst, 0).unwrap()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A general goal; this is the full range of questions you can pose to Chalk.
pub enum Goal {
    /// Introduces a binding at depth 0, shifting other bindings up
    /// (deBruijn index).
    Quantified(QuantifierKind, Binders<Box<Goal>>),
    Implies(Vec<DomainGoal>, Box<Goal>),
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
        Goal::Quantified(kind, Binders { value: Box::new(self), binders })
    }

    pub fn implied_by(self, predicates: Vec<DomainGoal>) -> Goal {
        Goal::Implies(predicates, Box::new(self))
    }

    /// Returns a canonical goal in which the outermost `exists<>` and
    /// `forall<>` quantifiers (as well as implications) have been
    /// "peeled" and are converted into free universal or existential
    /// variables. Assumes that this goal is a "closed goal" which
    /// does not -- at present -- contain any variables. Useful for
    /// REPLs and tests but not much else.
    pub fn into_peeled_goal(self) -> Canonical<InEnvironment<Goal>> {
        use solve::infer::InferenceTable;
        let mut infer = InferenceTable::new();
        let peeled_goal = {
            let mut env_goal = InEnvironment::new(&Environment::new(), self);
            loop {
                let InEnvironment { environment, goal } = env_goal;
                match goal {
                    Goal::Quantified(QuantifierKind::ForAll, subgoal) => {
                        let InEnvironment { environment, goal } =
                            subgoal.instantiate_universally(&environment);
                        env_goal = InEnvironment::new(&environment, *goal);
                    }

                    Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                        let subgoal = infer.instantiate_in(
                            environment.universe,
                            subgoal.binders.iter().cloned(),
                            &subgoal.value,
                        );
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
        infer.canonicalize(&peeled_goal).quantified
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QuantifierKind {
    ForAll, Exists
}

/// A constraint on lifetimes.
///
/// When we search for solutions within the trait system, we essentially ignore
/// lifetime constraints, instead gathering them up to return with our solution
/// for later checking. This allows for decoupling between type and region
/// checking in the compiler.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Constraint {
    LifetimeEq(Lifetime, Lifetime),
}

/// A mapping of inference variables to instantiations thereof.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Substitution {
    // Use BTreeMap for extracting in order (mostly for debugging/testing)
    pub tys: BTreeMap<InferenceVariable, Ty>,
    pub lifetimes: BTreeMap<InferenceVariable, Lifetime>,
}

impl Substitution {
    pub fn empty() -> Substitution {
        Substitution {
            tys: BTreeMap::new(),
            lifetimes: BTreeMap::new(),
        }
    }

    /// Construct an identity substitution given a set of binders
    pub fn from_binders(binders: &[ParameterKind<UniverseIndex>]) -> Self {
        let mut tys = BTreeMap::new();
        let mut lifetimes = BTreeMap::new();

        for (i, kind) in binders.iter().enumerate() {
            match *kind {
                ParameterKind::Ty(_) => {
                    tys.insert(InferenceVariable::from_depth(i), Ty::Var(i));
                }
                ParameterKind::Lifetime(_) => {
                    lifetimes.insert(InferenceVariable::from_depth(i), Lifetime::Var(i));
                }
            }
        }

        Substitution { tys, lifetimes }
    }

    pub fn is_empty(&self) -> bool {
        self.tys.is_empty() && self.lifetimes.is_empty()
    }
}

impl<'a> DefaultTypeFolder for &'a Substitution {
}

impl<'a> ExistentialFolder for &'a Substitution {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> ::errors::Result<Ty> {
        let v = InferenceVariable::from_depth(depth);
        if let Some(ty) = self.tys.get(&v) {
            // Substitutions do not have to be complete.
            Ok(ty.up_shift(binders))
        } else {
            Ok(Ty::Var(depth + binders))
        }
    }

    fn fold_free_existential_lifetime(&mut self, depth: usize, binders: usize) -> ::errors::Result<Lifetime> {
        let v = InferenceVariable::from_depth(depth);
        if let Some(l) = self.lifetimes.get(&v) {
            // Substitutions do not have to be complete.
            Ok(l.up_shift(binders))
        } else {
            Ok(Lifetime::Var(depth + binders))
        }
    }
}

impl<'a> IdentityUniversalFolder for &'a Substitution {
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstrainedSubst {
    pub subst: Substitution,
    pub constraints: Vec<InEnvironment<Constraint>>,
}

pub mod debug;
mod tls;

pub use self::tls::set_current_program;
pub use self::tls::with_current_program;
