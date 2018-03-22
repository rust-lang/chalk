use crate::fallible::Fallible;
use crate::{ExClause, SimplifiedAnswer};
use crate::hh::HhGoal;
use std::fmt::Debug;
use std::hash::Hash;

crate mod prelude;

pub trait Context
    : Sized + Clone + Debug + ContextOps<Self> + Aggregate<Self>
{
    /// Represents a set of hypotheses that are assumed to be true.
    type Environment: Environment<Self>;

    /// Goals correspond to things we can prove.
    type Goal: Goal<Self>;

    /// A goal that can be targeted by a program clause. The SLG
    /// solver treats these opaquely; in contrast, it understands
    /// "meta" goals like `G1 && G2` and so forth natively.
    type DomainGoal: DomainGoal<Self>;

    /// A map between universes. These are produced when
    /// u-canonicalizing something; they map canonical results back to
    /// the universes from the original.
    type UniverseMap: UniverseMap<Self>;

    /// Represents a goal along with an environment.
    type GoalInEnvironment: GoalInEnvironment<Self>;

    type CanonicalExClause: Debug;

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

    /// Represents a region constraint that will be propagated back
    /// (but not verified).
    type RegionConstraint: Debug;

    /// Represents a substitution from the "canonical variables" found
    /// in a canonical goal to specific values.
    type Substitution: Debug;

    /// Part of an answer: represents a canonicalized substitution,
    /// combined with region constraints. See [the rustc-guide] for more information.
    ///
    /// [the rustc-guide]: https://rust-lang-nursery.github.io/rustc-guide/traits-canonicalization.html#canonicalizing-the-query-result
    type CanonicalConstrainedSubst: CanonicalConstrainedSubst<Self>;

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
    type UnificationResult: UnificationResult<Self>;

    /// A final solution that is passed back to the user. This is
    /// completely opaque to the SLG solver; it is produced by
    /// `make_solution`.
    type Solution;
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
pub trait TruncateOps<C: Context> {
    /// If `subgoal` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_goal(
        &mut self,
        subgoal: &C::GoalInEnvironment,
    ) -> Option<C::GoalInEnvironment>;

    /// If `subst` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_answer(
        &mut self,
        subst: &C::Substitution,
    ) -> Option<C::Substitution>;
}

pub trait ContextOps<C: Context> {
    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &C::UCanonicalGoalInEnvironment) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    fn program_clauses(
        &self,
        environment: &C::Environment,
        goal: &C::DomainGoal,
    ) -> Vec<C::ProgramClause>;

    fn goal_in_environment(environment: &C::Environment, goal: C::Goal) -> C::GoalInEnvironment;

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
        op: impl FnOnce(&mut dyn InferenceTable<C>, C::Substitution, C::Environment, C::Goal) -> R,
    ) -> R;

    fn instantiate_ex_clause<R>(
        &self,
        num_universes: usize,
        canonical_ex_clause: &C::CanonicalExClause,
        op: impl FnOnce(&mut dyn InferenceTable<C>, ExClause<C>) -> R
    ) -> R;
}

pub trait ResolventOps<C: Context> {
    /// Combines the `goal` (instantiated within `infer`) with the
    /// given program clause to yield the start of a new strand (a
    /// canonical ex-clause).
    ///
    /// The bindings in `infer` are unaffected by this operation.
    fn resolvent_clause(
        &mut self,
        environment: &C::Environment,
        goal: &C::DomainGoal,
        subst: &C::Substitution,
        clause: &C::ProgramClause,
    ) -> Fallible<C::CanonicalExClause>;

    fn apply_answer_subst(
        &mut self,
        ex_clause: ExClause<C>,
        selected_goal: &C::GoalInEnvironment,
        answer_table_goal: &C::CanonicalGoalInEnvironment,
        canonical_answer_subst: &C::CanonicalConstrainedSubst,
    ) -> Fallible<ExClause<C>>;
}

pub trait Aggregate<C: Context> {
    fn make_solution(
        &self,
        root_goal: &C::CanonicalGoalInEnvironment,
        simplified_answers: impl IntoIterator<Item = SimplifiedAnswer<C>>,
    ) -> Option<C::Solution>;
}

pub trait UCanonicalGoalInEnvironment<C: Context>: Debug + Clone + Eq + Hash {
    fn canonical(&self) -> &C::CanonicalGoalInEnvironment;
    fn is_trivial_substitution(&self, canonical_subst: &C::CanonicalConstrainedSubst) -> bool;
    fn num_universes(&self) -> usize;
}

pub trait GoalInEnvironment<C: Context>: Debug + Clone + Eq + Ord + Hash {
    fn environment(&self) -> &C::Environment;
}

pub trait Environment<C: Context>: Debug + Clone {
    // Used by: simplify
    fn add_clauses(&self, clauses: impl IntoIterator<Item = C::DomainGoal>) -> Self;
}

pub trait InferenceTable<C: Context>: ResolventOps<C> + TruncateOps<C> {
    // Used by: simplify
    fn instantiate_binders_universally(&mut self, arg: &C::BindersGoal) -> C::Goal;

    // Used by: simplify
    fn instantiate_binders_existentially(&mut self, arg: &C::BindersGoal) -> C::Goal;

    // Used by: logic (but for debugging only)
    fn debug_ex_clause(&mut self, value: &'v ExClause<C>) -> Box<dyn Debug + 'v>;

    // Used by: logic
    fn canonicalize_goal(&mut self, value: &C::GoalInEnvironment) -> C::CanonicalGoalInEnvironment;

    // Used by: logic
    fn canonicalize_ex_clause(&mut self, value: &ExClause<C>) -> C::CanonicalExClause;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        subst: C::Substitution,
        constraints: Vec<C::RegionConstraint>,
    ) -> C::CanonicalConstrainedSubst;

    // Used by: logic
    fn u_canonicalize_goal(
        &mut self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> (C::UCanonicalGoalInEnvironment, C::UniverseMap);

    // Used by: logic
    fn invert_goal(&mut self, value: &C::GoalInEnvironment) -> Option<C::GoalInEnvironment>;

    // Used by: simplify
    fn unify_parameters(
        &mut self,
        environment: &C::Environment,
        a: &C::Parameter,
        b: &C::Parameter,
    ) -> Fallible<C::UnificationResult>;
}

pub trait CanonicalConstrainedSubst<C: Context>: Clone + Debug + Eq + Hash + Ord {
    fn empty_constraints(&self) -> bool;
}

pub trait DomainGoal<C: Context>: Debug {
    fn into_goal(self) -> C::Goal;
}

pub trait Goal<C: Context>: Clone + Debug + Eq {
    fn cannot_prove() -> Self;
    fn into_hh_goal(self) -> HhGoal<C>;
}

pub trait UniverseMap<C: Context>: Clone + Debug {
    fn map_goal_from_canonical(
        &self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment;

    fn map_subst_from_canonical(
        &self,
        value: &C::CanonicalConstrainedSubst,
    ) -> C::CanonicalConstrainedSubst;
}

pub trait UnificationResult<C: Context> {
    fn into_ex_clause(self, ex_clause: &mut ExClause<C>);
}
