//! Defines traits used to embed the chalk-engine in another crate.
//!
//! chalk and rustc both define types which implement the traits in this
//! module. This allows each user of chalk-engine to define their own
//! `DomainGoal` type, add arena lifetime parameters, and more. See
//! [`Context`] trait for a list of types.

use crate::fallible::Fallible;
use crate::hh::HhGoal;
use crate::{CompleteAnswer, ExClause};
use std::fmt::Debug;
use std::hash::Hash;

/// The "context" in which the SLG solver operates. It defines all the
/// types that the SLG solver may need to refer to, as well as a few
/// very simple interconversion methods.
///
/// At any given time, the SLG solver may have more than one context
/// active. First, there is always the *global* context, but when we
/// are in the midst of pursuing some particular strand, we will
/// instantiate a second context just for that work, via the
/// `instantiate_ucanonical_goal` and `instantiate_ex_clause` methods.
///
/// In the chalk implementation, these two contexts are mapped to the
/// same type. But in the rustc implementation, this second context
/// corresponds to a fresh arena, and data allocated in that second
/// context will be freed once the work is done. (The "canonicalizing"
/// steps offer a way to convert data from the inference context back
/// into the global context.)
///
/// FIXME: Clone and Debug bounds are just for easy derive, they are
/// not actually necessary. But dang are they convenient.
pub trait Context: Clone + Debug {
    type CanonicalExClause: Debug;

    /// A map between universes. These are produced when
    /// u-canonicalizing something; they map canonical results back to
    /// the universes from the original.
    type UniverseMap: Clone + Debug;

    /// Extracted from a canonicalized substitution or canonicalized ex clause, this is the type of
    /// substitution that is fully normalized with respect to inference variables.
    type InferenceNormalizedSubst: Debug;

    /// A canonicalized `GoalInEnvironment` -- that is, one where all
    /// free inference variables have been bound into the canonical
    /// binder. See [the rustc-dev-guide] for more information.
    ///
    /// [the rustc-dev-guide]: https://rustc-dev-guide.rust-lang.org/traits/canonicalization.html
    type CanonicalGoalInEnvironment: Debug;

    /// A u-canonicalized `GoalInEnvironment` -- this is one where the
    /// free universes are renumbered to consecutive integers starting
    /// from U1 (but preserving their relative order).
    type UCanonicalGoalInEnvironment: Debug + Clone + Eq + Hash;

    /// A final solution that is passed back to the user. This is
    /// completely opaque to the SLG solver; it is produced by
    /// `make_solution`.
    type Solution;

    /// Part of a complete answer: the canonical version of a
    /// substitution and region constraints but without any delayed
    /// literals.
    type CanonicalConstrainedSubst: Clone + Debug + Eq + Hash;

    /// Part of an answer: the canonical version of a substitution,
    /// region constraints, and delayed literals.
    type CanonicalAnswerSubst: Clone + Debug + Eq + Hash;

    /// Represents a substitution from the "canonical variables" found
    /// in a canonical goal to specific values.
    type Substitution: Clone + Debug;

    /// Represents a region constraint that will be propagated back
    /// (but not verified).
    type RegionConstraint: Clone + Debug;

    /// Represents a goal along with an environment.
    type GoalInEnvironment: Debug + Clone + Eq + Hash;

    /// Represents an inference table.
    type InferenceTable: InferenceTable<Self> + Clone;

    /// Represents a set of hypotheses that are assumed to be true.
    type Environment: Debug + Clone;

    /// Goals correspond to things we can prove.
    type Goal: Clone + Debug + Eq;

    /// A goal that can be targeted by a program clause. The SLG
    /// solver treats these opaquely; in contrast, it understands
    /// "meta" goals like `G1 && G2` and so forth natively.
    type DomainGoal: Debug;

    /// A "higher-order" goal, quantified over some types and/or
    /// lifetimes. When you have a quantification, like `forall<T> { G
    /// }` or `exists<T> { G }`, this represents the `<T> { G }` part.
    ///
    /// (In Lambda Prolog, this would be a "lambda predicate", like `T
    /// \ Goal`).
    type BindersGoal: Debug;

    /// A term that can be quantified over and unified -- in current
    /// Chalk, either a type or lifetime.
    type Parameter: Debug;

    /// A rule like `DomainGoal :- Goal`.
    ///
    /// `resolvent_clause` combines a program-clause and a concrete
    /// goal we are trying to solve to produce an ex-clause.
    type ProgramClause: Debug;

    /// A vector of program clauses.
    type ProgramClauses: Debug;

    /// How to relate two kinds when unifying: for example in rustc, we
    /// may want to unify parameters either for the sub-typing relation or for
    /// the equality relation.
    type Variance;

    /// The type used to store concrete representations of "core types" from chalk-ir.
    type Interner;

    /// Given an environment and a goal, glue them together to create
    /// a `GoalInEnvironment`.
    fn goal_in_environment(
        environment: &Self::Environment,
        goal: Self::Goal,
    ) -> Self::GoalInEnvironment;

    /// Extracts the inner normalized substitution from a canonical ex-clause.
    fn inference_normalized_subst_from_ex_clause(
        canon_ex_clause: &Self::CanonicalExClause,
    ) -> &Self::InferenceNormalizedSubst;

    /// Extracts the inner normalized substitution from a canonical constraint subst.
    fn inference_normalized_subst_from_subst(
        canon_ex_clause: &Self::CanonicalAnswerSubst,
    ) -> &Self::InferenceNormalizedSubst;

    /// True if this solution has no region constraints.
    fn empty_constraints(ccs: &Self::CanonicalAnswerSubst) -> bool;

    fn canonical(u_canon: &Self::UCanonicalGoalInEnvironment) -> &Self::CanonicalGoalInEnvironment;

    fn has_delayed_subgoals(canonical_subst: &Self::CanonicalAnswerSubst) -> bool;

    fn num_universes(_: &Self::UCanonicalGoalInEnvironment) -> usize;

    fn canonical_constrained_subst_from_canonical_constrained_answer(
        canonical_subst: &Self::CanonicalAnswerSubst,
    ) -> Self::CanonicalConstrainedSubst;

    fn goal_from_goal_in_environment(goal: &Self::GoalInEnvironment) -> &Self::Goal;

    /// Selects the next appropriate subgoal index for evaluation.
    /// Used by: logic
    fn next_subgoal_index(ex_clause: &ExClause<Self>) -> usize;
}

pub trait ContextOps<C: Context>: Sized + Clone + Debug + AggregateOps<C> {
    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &C::UCanonicalGoalInEnvironment) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    ///
    /// If this callback returns `None`, that indicates that the set
    /// of program clauses cannot be enumerated because there are
    /// unresolved type variables that would have to be resolved
    /// first; the goal will be considered floundered.
    fn program_clauses(
        &self,
        environment: &C::Environment,
        goal: &C::DomainGoal,
        infer: &mut C::InferenceTable,
    ) -> Result<Vec<C::ProgramClause>, Floundered>;

    // Used by: simplify
    fn add_clauses(&self, env: &C::Environment, clauses: C::ProgramClauses) -> C::Environment;

    /// Create an inference table for processing a new goal and instantiate that goal
    /// in that context, returning "all the pieces".
    ///
    /// More specifically: given a u-canonical goal `arg`, creates a
    /// new inference table `T` and populates it with the universes
    /// found in `arg`. Then, creates a substitution `S` that maps
    /// each bound variable in `arg` to a fresh inference variable
    /// from T. Returns:
    ///
    /// - the table `T`
    /// - the substitution `S`
    /// - the environment and goal found by substitution `S` into `arg`
    fn instantiate_ucanonical_goal(
        &self,
        arg: &C::UCanonicalGoalInEnvironment,
    ) -> (C::InferenceTable, C::Substitution, C::Environment, C::Goal);

    fn instantiate_ex_clause(
        &self,
        num_universes: usize,
        canonical_ex_clause: &C::CanonicalExClause,
    ) -> (C::InferenceTable, ExClause<C>);

    // Used by: logic
    fn instantiate_answer_subst(
        &self,
        num_universes: usize,
        answer: &C::CanonicalAnswerSubst,
    ) -> (
        C::InferenceTable,
        C::Substitution,
        Vec<C::RegionConstraint>,
        Vec<C::GoalInEnvironment>,
    );

    /// returns unique solution from answer
    fn constrained_subst_from_answer(
        &self,
        answer: CompleteAnswer<C>,
    ) -> C::CanonicalConstrainedSubst;

    /// Returns a identity substitution.
    fn identity_constrained_subst(
        &self,
        goal: &C::UCanonicalGoalInEnvironment,
    ) -> C::CanonicalConstrainedSubst;

    /// Convert a goal G *from* the canonical universes *into* our
    /// local universes. This will yield a goal G' that is the same
    /// but for the universes of universally quantified names.
    fn map_goal_from_canonical(
        &self,
        _: &C::UniverseMap,
        value: &C::CanonicalGoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment;

    /// Convert a substitution *from* the canonical universes *into*
    /// our local universes. This will yield a substitution S' that is
    /// the same but for the universes of universally quantified
    /// names.
    fn map_subst_from_canonical(
        &self,
        _: &C::UniverseMap,
        value: &C::CanonicalAnswerSubst,
    ) -> C::CanonicalAnswerSubst;

    fn interner(&self) -> &C::Interner;

    /// Upcast this domain goal into a more general goal.
    fn into_goal(&self, domain_goal: C::DomainGoal) -> C::Goal;

    fn is_trivial_substitution(
        &self,
        u_canon: &C::UCanonicalGoalInEnvironment,
        canonical_subst: &C::CanonicalAnswerSubst,
    ) -> bool;

    /// Convert the context's goal type into the `HhGoal` type that
    /// the SLG solver understands. The expectation is that the
    /// context's goal type has the same set of variants, but with
    /// different names and a different setup. If you inspect
    /// `HhGoal`, you will see that this is a "shallow" or "lazy"
    /// conversion -- that is, we convert the outermost goal into an
    /// `HhGoal`, but the goals contained within are left as context
    /// goals.
    fn into_hh_goal(&self, goal: C::Goal) -> HhGoal<C>;
}

/// Methods for combining solutions to yield an aggregate solution.
pub trait AggregateOps<C: Context> {
    fn make_solution(
        &self,
        root_goal: &C::UCanonicalGoalInEnvironment,
        answers: impl AnswerStream<C>,
        should_continue: impl Fn() -> bool,
    ) -> Option<C::Solution>;
}

/// An "inference table" contains the state to support unification and
/// other operations on terms.
pub trait InferenceTable<C: Context>: ResolventOps<C> + TruncateOps<C> + UnificationOps<C> {}

/// Error type for the `UnificationOps::program_clauses` method --
/// indicates that the complete set of program clauses for this goal
/// cannot be enumerated.
pub struct Floundered;

/// Methods for unifying and manipulating terms and binders.
pub trait UnificationOps<C: Context> {
    // Used by: simplify
    fn instantiate_binders_universally(
        &mut self,
        interner: &C::Interner,
        arg: &C::BindersGoal,
    ) -> C::Goal;

    // Used by: simplify
    fn instantiate_binders_existentially(
        &mut self,
        interner: &C::Interner,
        arg: &C::BindersGoal,
    ) -> C::Goal;

    // Used by: logic (but for debugging only)
    fn debug_ex_clause<'v>(
        &mut self,
        interner: &C::Interner,
        value: &'v ExClause<C>,
    ) -> Box<dyn Debug + 'v>;

    // Used by: logic
    fn fully_canonicalize_goal(
        &mut self,
        interner: &C::Interner,
        value: &C::GoalInEnvironment,
    ) -> (C::UCanonicalGoalInEnvironment, C::UniverseMap);

    // Used by: logic
    fn canonicalize_ex_clause(
        &mut self,
        interner: &C::Interner,
        value: &ExClause<C>,
    ) -> C::CanonicalExClause;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        interner: &C::Interner,
        subst: C::Substitution,
        constraints: Vec<C::RegionConstraint>,
    ) -> C::CanonicalConstrainedSubst;

    // Used by: logic
    fn canonicalize_answer_subst(
        &mut self,
        interner: &C::Interner,
        subst: C::Substitution,
        constraints: Vec<C::RegionConstraint>,
        delayed_subgoals: Vec<C::GoalInEnvironment>,
    ) -> C::CanonicalAnswerSubst;

    // Used by: logic
    fn invert_goal(
        &mut self,
        interner: &C::Interner,
        value: &C::GoalInEnvironment,
    ) -> Option<C::GoalInEnvironment>;

    /// First unify the parameters, then add the residual subgoals
    /// as new subgoals of the ex-clause.
    /// Also add region constraints.
    ///
    /// If the parameters fail to unify, then `Error` is returned
    // Used by: simplify
    fn unify_parameters_into_ex_clause(
        &mut self,
        interner: &C::Interner,
        environment: &C::Environment,
        variance: C::Variance,
        a: &C::Parameter,
        b: &C::Parameter,
        ex_clause: &mut ExClause<C>,
    ) -> Fallible<()>;
}

/// "Truncation" (called "abstraction" in the papers referenced below)
/// refers to the act of modifying a goal or answer that has become
/// too large in order to guarantee termination.
///
/// Currently we don't perform truncation (but it might me readded later).
///
/// Citations:
///
/// - Terminating Evaluation of Logic Programs with Finite Three-Valued Models
///   - Riguzzi and Swift; ACM Transactions on Computational Logic 2013
/// - Radial Restraint
///   - Grosof and Swift; 2013
pub trait TruncateOps<C: Context> {
    /// Check if `subgoal` is too large
    fn goal_needs_truncation(
        &mut self,
        interner: &C::Interner,
        subgoal: &C::GoalInEnvironment,
    ) -> bool;

    /// Check if `subst` is too large
    fn answer_needs_truncation(&mut self, interner: &C::Interner, subst: &C::Substitution) -> bool;
}

pub trait ResolventOps<C: Context> {
    /// Combines the `goal` (instantiated within `infer`) with the
    /// given program clause to yield the start of a new strand (a
    /// canonical ex-clause).
    ///
    /// The bindings in `infer` are unaffected by this operation.
    fn resolvent_clause(
        &mut self,
        interner: &C::Interner,
        environment: &C::Environment,
        goal: &C::DomainGoal,
        subst: &C::Substitution,
        clause: &C::ProgramClause,
    ) -> Fallible<ExClause<C>>;

    fn apply_answer_subst(
        &mut self,
        interner: &C::Interner,
        ex_clause: &mut ExClause<C>,
        selected_goal: &C::GoalInEnvironment,
        answer_table_goal: &C::CanonicalGoalInEnvironment,
        canonical_answer_subst: &C::CanonicalAnswerSubst,
    ) -> Fallible<()>;
}

pub enum AnswerResult<C: Context> {
    /// The next available answer.
    Answer(CompleteAnswer<C>),

    /// No answer could be returned because there are no more solutions.
    NoMoreSolutions,

    /// No answer could be returned because the goal has floundered.
    Floundered,

    // No answer could be returned *yet*, because we exceeded our
    // quantum (`should_continue` returned false).
    QuantumExceeded,
}

impl<C: Context> AnswerResult<C> {
    pub fn is_answer(&self) -> bool {
        match self {
            Self::Answer(_) => true,
            _ => false,
        }
    }

    pub fn answer(self) -> CompleteAnswer<C> {
        match self {
            Self::Answer(answer) => answer,
            _ => panic!("Not an answer."),
        }
    }

    pub fn is_no_more_solutions(&self) -> bool {
        match self {
            Self::NoMoreSolutions => true,
            _ => false,
        }
    }

    pub fn is_quantum_exceeded(&self) -> bool {
        match self {
            Self::QuantumExceeded => true,
            _ => false,
        }
    }
}

impl<C: Context> Debug for AnswerResult<C> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnswerResult::Answer(answer) => write!(fmt, "{:?}", answer),
            AnswerResult::Floundered => write!(fmt, "Floundered"),
            AnswerResult::NoMoreSolutions => write!(fmt, "None"),
            AnswerResult::QuantumExceeded => write!(fmt, "QuantumExceeded"),
        }
    }
}

pub trait AnswerStream<C: Context> {
    /// Gets the next answer for a given goal, but doesn't increment the answer index.
    /// Calling this or `next_answer` again will give the same answer.
    fn peek_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<C>;

    /// Gets the next answer for a given goal, incrementing the answer index.
    /// Calling this or `peek_answer` again will give the next answer.
    fn next_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<C>;

    /// Invokes `test` with each possible future answer, returning true immediately
    /// if we find any answer for which `test` returns true.
    fn any_future_answer(&self, test: impl Fn(&C::InferenceNormalizedSubst) -> bool) -> bool;
}
