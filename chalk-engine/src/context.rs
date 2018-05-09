use crate::fallible::Fallible;
use crate::hh::HhGoal;
use crate::{ExClause, SimplifiedAnswer};
use std::fmt::Debug;
use std::hash::Hash;

crate mod prelude;

/// The "context" in which the SLG solver operates.
pub trait Context: Sized + Clone + Debug + ContextOps<Self> + AggregateOps<Self> {
    type CanonicalExClause: Debug;

    /// A map between universes. These are produced when
    /// u-canonicalizing something; they map canonical results back to
    /// the universes from the original.
    type UniverseMap: UniverseMap<Self>;

    /// Part of an answer: represents a canonicalized substitution,
    /// combined with region constraints. See [the rustc-guide] for more information.
    ///
    /// [the rustc-guide]: https://rust-lang-nursery.github.io/rustc-guide/traits-canonicalization.html#canonicalizing-the-query-result
    type CanonicalConstrainedSubst: Clone + Debug + Eq + Hash + Ord;

    /// Extracted from a canonicalized substitution or canonicalized ex clause, this is the type of
    /// substitution that is fully normalized with respect to inference variables.
    type InferenceNormalizedSubst: Debug;

    /// A canonicalized `GoalInEnvironment` -- that is, one where all
    /// free inference variables have been bound into the canonical
    /// binder. See [the rustc-guide] for more information.
    ///
    /// [the rustc-guide]: https://rust-lang-nursery.github.io/rustc-guide/traits-canonicalization.html
    type CanonicalGoalInEnvironment: Debug;

    /// A u-canonicalized `GoalInEnvironment` -- this is one where the
    /// free universes are renumbered to consecutive integers starting
    /// from U1 (but preserving their relative order).
    type UCanonicalGoalInEnvironment: UCanonicalGoalInEnvironment<Self>;

    /// A final solution that is passed back to the user. This is
    /// completely opaque to the SLG solver; it is produced by
    /// `make_solution`.
    type Solution;
}

pub trait ExClauseContext<C: Context>: Sized + Debug {
    /// Represents a substitution from the "canonical variables" found
    /// in a canonical goal to specific values.
    type Substitution: Debug;

    /// Represents a region constraint that will be propagated back
    /// (but not verified).
    type RegionConstraint: Debug;

    /// Represents a goal along with an environment.
    type GoalInEnvironment: Debug + Clone + Eq + Hash;
}

/// The set of types belonging to an "inference context"; in rustc,
/// these types are tied to the lifetime of the arena within which an
/// inference context operates.
pub trait InferenceContext<C: Context>: ExClauseContext<C> {
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

    /// The successful result from unification: contains new subgoals
    /// and things that can be attached to an ex-clause.
    type UnificationResult;

    fn goal_in_environment(
        environment: &Self::Environment,
        goal: Self::Goal,
    ) -> Self::GoalInEnvironment;

    /// Upcast this domain goal into a more general goal.
    fn into_goal(domain_goal: Self::DomainGoal) -> Self::Goal;

    /// Create a "cannot prove" goal (see `HhGoal::CannotProve`).
    fn cannot_prove() -> Self::Goal;

    /// Convert the context's goal type into the `HhGoal` type that
    /// the SLG solver understands. The expectation is that the
    /// context's goal type has the same set of variants, but with
    /// different names and a different setup. If you inspect
    /// `HhGoal`, you will see that this is a "shallow" or "lazy"
    /// conversion -- that is, we convert the outermost goal into an
    /// `HhGoal`, but the goals contained within are left as context
    /// goals.
    fn into_hh_goal(goal: Self::Goal) -> HhGoal<C, Self>;

    /// Add the residual subgoals as new subgoals of the ex-clause.
    /// Also add region constraints.
    fn into_ex_clause(result: Self::UnificationResult, ex_clause: &mut ExClause<C, Self>);

    // Used by: simplify
    fn add_clauses(
        env: &Self::Environment,
        clauses: impl IntoIterator<Item = Self::ProgramClause>,
    ) -> Self::Environment;
}

pub trait ContextOps<C: Context> {
    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &C::UCanonicalGoalInEnvironment) -> bool;

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
    fn instantiate_ucanonical_goal<R>(
        &self,
        arg: &C::UCanonicalGoalInEnvironment,
        op: impl WithInstantiatedUCanonicalGoal<C, Output = R>,
    ) -> R;

    fn instantiate_ex_clause<R>(
        &self,
        num_universes: usize,
        canonical_ex_clause: &C::CanonicalExClause,
        op: impl WithInstantiatedExClause<C, Output = R>,
    ) -> R;

    /// Extracts the inner normalized substitution from a canonical ex-clause.
    fn inference_normalized_subst_from_ex_clause(
        canon_ex_clause: &C::CanonicalExClause,
    ) -> &C::InferenceNormalizedSubst;

    /// Extracts the inner normalized substitution from a canonical constraint subst.
    fn inference_normalized_subst_from_subst(
        canon_ex_clause: &C::CanonicalConstrainedSubst,
    ) -> &C::InferenceNormalizedSubst;

    /// True if this solution has no region constraints.
    fn empty_constraints(ccs: &C::CanonicalConstrainedSubst) -> bool;
}

/// Callback trait for `instantiate_ucanonical_goal`. Unlike the other
/// traits in this file, this is not implemented by the context crate, but rather
/// by code in this crate.
///
/// This basically plays the role of an `FnOnce` -- but unlike an
/// `FnOnce`, the `with` method is generic.
pub trait WithInstantiatedUCanonicalGoal<C: Context> {
    type Output;

    fn with<I: InferenceContext<C>>(
        self,
        infer: &mut dyn InferenceTable<C, I>,
        subst: I::Substitution,
        environment: I::Environment,
        goal: I::Goal,
    ) -> Self::Output;
}

/// Callback trait for `instantiate_ex_clause`. Unlike the other
/// traits in this file, this is not implemented by the context crate,
/// but rather by code in this crate.
///
/// This basically plays the role of an `FnOnce` -- but unlike an
/// `FnOnce`, the `with` method is generic.
pub trait WithInstantiatedExClause<C: Context> {
    type Output;

    fn with<I: InferenceContext<C>>(
        self,
        infer: &mut dyn InferenceTable<C, I>,
        ex_clause: ExClause<C, I>,
    ) -> Self::Output;
}

/// Methods for combining solutions to yield an aggregate solution.
pub trait AggregateOps<C: Context> {
    fn make_solution(
        &self,
        root_goal: &C::CanonicalGoalInEnvironment,
        simplified_answers: impl AnswerStream<C>,
    ) -> Option<C::Solution>;
}

pub trait UCanonicalGoalInEnvironment<C: Context>: Debug + Clone + Eq + Hash {
    fn canonical(&self) -> &C::CanonicalGoalInEnvironment;
    fn is_trivial_substitution(&self, canonical_subst: &C::CanonicalConstrainedSubst) -> bool;
    fn num_universes(&self) -> usize;
}

/// An "inference table" contains the state to support unification and
/// other operations on terms.
pub trait InferenceTable<C: Context, I: InferenceContext<C>>:
    ResolventOps<C, I> + TruncateOps<C, I> + UnificationOps<C, I>
{
}

/// Methods for unifying and manipulating terms and binders.
pub trait UnificationOps<C: Context, I: InferenceContext<C>> {
    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    fn program_clauses(
        &self,
        environment: &I::Environment,
        goal: &I::DomainGoal,
    ) -> Vec<I::ProgramClause>;

    // Used by: simplify
    fn instantiate_binders_universally(&mut self, arg: &I::BindersGoal) -> I::Goal;

    // Used by: simplify
    fn instantiate_binders_existentially(&mut self, arg: &I::BindersGoal) -> I::Goal;

    // Used by: logic (but for debugging only)
    fn debug_ex_clause(&mut self, value: &'v ExClause<C, I>) -> Box<dyn Debug + 'v>;

    // Used by: logic
    fn canonicalize_goal(&mut self, value: &I::GoalInEnvironment) -> C::CanonicalGoalInEnvironment;

    // Used by: logic
    fn canonicalize_ex_clause(&mut self, value: &ExClause<C, I>) -> C::CanonicalExClause;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        subst: I::Substitution,
        constraints: Vec<I::RegionConstraint>,
    ) -> C::CanonicalConstrainedSubst;

    // Used by: logic
    fn u_canonicalize_goal(
        &mut self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> (C::UCanonicalGoalInEnvironment, C::UniverseMap);

    // Used by: logic
    fn invert_goal(&mut self, value: &I::GoalInEnvironment) -> Option<I::GoalInEnvironment>;

    // Used by: simplify
    fn unify_parameters(
        &mut self,
        environment: &I::Environment,
        a: &I::Parameter,
        b: &I::Parameter,
    ) -> Fallible<I::UnificationResult>;
}

/// "Truncation" (called "abstraction" in the papers referenced below)
/// refers to the act of modifying a goal or answer that has become
/// too large in order to guarantee termination. The SLG solver
/// doesn't care about the precise truncation function, so long as
/// it's deterministic and so forth.
///
/// Citations:
///
/// - Terminating Evaluation of Logic Programs with Finite Three-Valued Models
///   - Riguzzi and Swift; ACM Transactions on Computational Logic 2013
/// - Radial Restraint
///   - Grosof and Swift; 2013
pub trait TruncateOps<C: Context, I: InferenceContext<C>> {
    /// If `subgoal` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_goal(&mut self, subgoal: &I::GoalInEnvironment) -> Option<I::GoalInEnvironment>;

    /// If `subst` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_answer(&mut self, subst: &I::Substitution) -> Option<I::Substitution>;
}

pub trait ResolventOps<C: Context, I: InferenceContext<C>> {
    /// Combines the `goal` (instantiated within `infer`) with the
    /// given program clause to yield the start of a new strand (a
    /// canonical ex-clause).
    ///
    /// The bindings in `infer` are unaffected by this operation.
    fn resolvent_clause(
        &mut self,
        environment: &I::Environment,
        goal: &I::DomainGoal,
        subst: &I::Substitution,
        clause: &I::ProgramClause,
    ) -> Fallible<C::CanonicalExClause>;

    fn apply_answer_subst(
        &mut self,
        ex_clause: ExClause<C, I>,
        selected_goal: &I::GoalInEnvironment,
        answer_table_goal: &C::CanonicalGoalInEnvironment,
        canonical_answer_subst: &C::CanonicalConstrainedSubst,
    ) -> Fallible<ExClause<C, I>>;
}

pub trait UniverseMap<C: Context>: Clone + Debug {
    /// Convert a goal G *from* the canonical universes *into* our
    /// local universes. This will yield a goal G' that is the same
    /// but for the universes of universally quantified names.
    fn map_goal_from_canonical(
        &self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment;

    /// Convert a substitution *from* the canonical universes *into*
    /// our local universes. This will yield a substitution S' that is
    /// the same but for the universes of universally quantified
    /// names.
    fn map_subst_from_canonical(
        &self,
        value: &C::CanonicalConstrainedSubst,
    ) -> C::CanonicalConstrainedSubst;
}

pub trait AnswerStream<C: Context> {
    fn peek_answer(&mut self) -> Option<SimplifiedAnswer<C>>;
    fn next_answer(&mut self) -> Option<SimplifiedAnswer<C>>;

    /// Invokes `test` with each possible future answer, returning true immediately
    /// if we find any answer for which `test` returns true.
    fn any_future_answer(&mut self, test: impl FnMut(&C::InferenceNormalizedSubst) -> bool)
        -> bool;
}
