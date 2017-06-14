use cast::Cast;
use chalk_parse::ast;
use lalrpop_intern::InternedString;
use solve::infer::{TyInferenceVariable, LifetimeInferenceVariable};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;

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

    /// For each trait:
    pub associated_ty_data: HashMap<ItemId, AssociatedTyDatum>,
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
        env.clauses.extend(clauses);
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
    pub trait_ref: TraitRef,
    pub where_clauses: Vec<DomainGoal>,
    pub associated_ty_values: Vec<AssociatedTyValue>,
    pub specialization_priority: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatum {
    pub binders: Binders<StructDatumBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StructDatumBound {
    pub self_ty: ApplicationTy,
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
    /// A clause of the form (T: Foo<Item = U>) expands to (T: Foo<Item = U>), (T: Foo), WF(T: Foo).
    pub fn expanded(self, program: &Program) -> impl Iterator<Item = DomainGoal> {
        let mut expanded = vec![];
        match self {
            DomainGoal::Implemented(ref trait_ref) => {
                expanded.push(WellFormed::TraitRef(trait_ref.clone()).cast());
            }
            DomainGoal::Normalize(Normalize { ref projection, .. }) => {
                let (associated_ty_data, trait_params, _) = program.split_projection(&projection);
                let trait_ref = TraitRef {
                    trait_id: associated_ty_data.trait_id,
                    parameters: trait_params.to_owned()
                };
                expanded.push(trait_ref.clone().cast());
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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

impl<T> Canonical<T> {
    pub fn map<OP, U>(self, op: OP) -> Canonical<U>
        where OP: FnOnce(T) -> U
    {
        Canonical { value: op(self.value), binders: self.binders }
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
}

impl Goal {
    pub fn quantify(self, kind: QuantifierKind, binders: Vec<ParameterKind<()>>) -> Goal {
        Goal::Quantified(kind, Binders { value: Box::new(self), binders })
    }

    pub fn implied_by(self, predicates: Vec<DomainGoal>) -> Goal {
        Goal::Implies(predicates, Box::new(self))
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substitution {
    // Use BTreeMap for extracting in order (mostly for debugging/testing)
    pub tys: BTreeMap<TyInferenceVariable, Ty>,
    pub lifetimes: BTreeMap<LifetimeInferenceVariable, Lifetime>,
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
                    tys.insert(TyInferenceVariable::from_depth(i), Ty::Var(i));
                }
                ParameterKind::Lifetime(_) => {
                    lifetimes.insert(LifetimeInferenceVariable::from_depth(i), Lifetime::Var(i));
                }
            }
        }

        Substitution { tys, lifetimes }
    }

    pub fn is_empty(&self) -> bool {
        self.tys.is_empty() && self.lifetimes.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstrainedSubst {
    pub subst: Substitution,
    pub constraints: Vec<InEnvironment<Constraint>>,
}

pub mod debug;
mod tls;

pub use self::tls::set_current_program;
pub use self::tls::with_current_program;
