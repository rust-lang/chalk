use fallible::Fallible;
use hh::HhGoal;
use {DelayedLiteral, ExClause, SimplifiedAnswer};
use std::fmt::Debug;
use std::hash::Hash;

pub(crate) mod prelude;

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
    /// binder. See [the rustc-guide] for more information.
    ///
    /// [the rustc-guide]: https://rust-lang-nursery.github.io/rustc-guide/traits-canonicalization.html
    type CanonicalGoalInEnvironment: Debug;

    /// A u-canonicalized `GoalInEnvironment` -- this is one where the
    /// free universes are renumbered to consecutive integers starting
    /// from U1 (but preserving their relative order).
    type UCanonicalGoalInEnvironment: Debug + Clone + Eq + Hash;

    /// A final solution that is passed back to the user. This is
    /// completely opaque to the SLG solver; it is produced by
    /// `make_solution`.
    type Solution;

    /// Part of an answer: represents a canonicalized substitution,
    /// combined with region constraints. See [the rustc-guide] for more information.
    ///
    /// [the rustc-guide]: https://rust-lang-nursery.github.io/rustc-guide/traits-canonicalization.html#canonicalizing-the-query-result
    type CanonicalConstrainedSubst: Clone + Debug + Eq + Hash;

    /// Represents a substitution from the "canonical variables" found
    /// in a canonical goal to specific values.
    type Substitution: Debug;

    /// Represents a region constraint that will be propagated back
    /// (but not verified).
    type RegionConstraint: Debug;

    /// Represents a goal along with an environment.
    type GoalInEnvironment: Debug + Clone + Eq + Hash;

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

    /// The successful result from unification: contains new subgoals
    /// and things that can be attached to an ex-clause.
    type UnificationResult;

    /// Given an environment and a goal, glue them together to create
    /// a `GoalInEnvironment`.
    fn goal_in_environment(
        environment: &Self::Environment,
        goal: Self::Goal,
    ) -> Self::GoalInEnvironment;
}

pub trait ContextOps<C: Context>: Sized + Clone + Debug + AggregateOps<C> {
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

    fn canonical(u_canon: &C::UCanonicalGoalInEnvironment) -> &C::CanonicalGoalInEnvironment;
    fn is_trivial_substitution(u_canon: &C::UCanonicalGoalInEnvironment,
                               canonical_subst: &C::CanonicalConstrainedSubst) -> bool;
    fn num_universes(&C::UCanonicalGoalInEnvironment) -> usize;

    /// Convert a goal G *from* the canonical universes *into* our
    /// local universes. This will yield a goal G' that is the same
    /// but for the universes of universally quantified names.
    fn map_goal_from_canonical(
        &C::UniverseMap,
        value: &C::CanonicalGoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment;

    /// Convert a substitution *from* the canonical universes *into*
    /// our local universes. This will yield a substitution S' that is
    /// the same but for the universes of universally quantified
    /// names.
    fn map_subst_from_canonical(
        &C::UniverseMap,
        value: &C::CanonicalConstrainedSubst,
    ) -> C::CanonicalConstrainedSubst;
}

/// Callback trait for `instantiate_ucanonical_goal`. Unlike the other
/// traits in this file, this is not implemented by the context crate, but rather
/// by code in this crate.
///
/// This basically plays the role of an `FnOnce` -- but unlike an
/// `FnOnce`, the `with` method is generic.
pub trait WithInstantiatedUCanonicalGoal<C: Context> {
    type Output;

    fn with<I: Context>(
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

    fn with<I: Context>(
        self,
        infer: &mut dyn InferenceTable<C, I>,
        ex_clause: ExClause<I>,
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

/// An "inference table" contains the state to support unification and
/// other operations on terms.
pub trait InferenceTable<C: Context, I: Context>:
    ResolventOps<C, I> + TruncateOps<C, I> + UnificationOps<C, I>
{
    /// Convert the context's goal type into the `HhGoal` type that
    /// the SLG solver understands. The expectation is that the
    /// context's goal type has the same set of variants, but with
    /// different names and a different setup. If you inspect
    /// `HhGoal`, you will see that this is a "shallow" or "lazy"
    /// conversion -- that is, we convert the outermost goal into an
    /// `HhGoal`, but the goals contained within are left as context
    /// goals.
    fn into_hh_goal(&mut self, goal: I::Goal) -> HhGoal<I>;

    // Used by: simplify
    fn add_clauses(
        &mut self,
        env: &I::Environment,
        clauses: I::ProgramClauses,
    ) -> I::Environment;

    /// Upcast this domain goal into a more general goal.
    fn into_goal(&self, domain_goal: I::DomainGoal) -> I::Goal;

    /// Create a "cannot prove" goal (see `HhGoal::CannotProve`).
    fn cannot_prove(&self) -> I::Goal;
}

/// Methods for unifying and manipulating terms and binders.
pub trait UnificationOps<C: Context, I: Context> {
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
    fn debug_ex_clause(&mut self, value: &'v ExClause<I>) -> Box<dyn Debug + 'v>;

    // Used by: logic
    fn canonicalize_goal(&mut self, value: &I::GoalInEnvironment) -> C::CanonicalGoalInEnvironment;

    // Used by: logic
    fn canonicalize_ex_clause(&mut self, value: &ExClause<I>) -> C::CanonicalExClause;

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

    fn sink_answer_subset(
        &self,
        value: &C::CanonicalConstrainedSubst,
    ) -> I::CanonicalConstrainedSubst;

    fn lift_delayed_literal(
        &self,
        value: DelayedLiteral<I>,
    ) -> DelayedLiteral<C>;

    // Used by: logic
    fn invert_goal(&mut self, value: &I::GoalInEnvironment) -> Option<I::GoalInEnvironment>;

    // Used by: simplify
    fn unify_parameters(
        &mut self,
        environment: &I::Environment,
        variance: I::Variance,
        a: &I::Parameter,
        b: &I::Parameter,
    ) -> Fallible<I::UnificationResult>;

    /// Add the residual subgoals as new subgoals of the ex-clause.
    /// Also add region constraints.
    fn into_ex_clause(&mut self, result: I::UnificationResult, ex_clause: &mut ExClause<I>);
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
pub trait TruncateOps<C: Context, I: Context> {
    /// If `subgoal` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_goal(&mut self, subgoal: &I::GoalInEnvironment) -> Option<I::GoalInEnvironment>;

    /// If `subst` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_answer(&mut self, subst: &I::Substitution) -> Option<I::Substitution>;
}

pub trait ResolventOps<C: Context, I: Context> {
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
        ex_clause: ExClause<I>,
        selected_goal: &I::GoalInEnvironment,
        answer_table_goal: &C::CanonicalGoalInEnvironment,
        canonical_answer_subst: &C::CanonicalConstrainedSubst,
    ) -> Fallible<ExClause<I>>;
}

pub trait AnswerStream<C: Context> {
    fn peek_answer(&mut self) -> Option<SimplifiedAnswer<C>>;
    fn next_answer(&mut self) -> Option<SimplifiedAnswer<C>>;

    /// Invokes `test` with each possible future answer, returning true immediately
    /// if we find any answer for which `test` returns true.
    fn any_future_answer(&mut self, test: impl FnMut(&C::InferenceNormalizedSubst) -> bool)
        -> bool;
}
