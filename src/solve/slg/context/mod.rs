use crate::fallible::Fallible;
use crate::solve::slg::{ExClause, Satisfiable, SimplifiedAnswer};
use crate::solve::slg::hh::HhGoal;
use std::fmt::Debug;
use std::hash::Hash;

crate mod implementation;
crate mod prelude;

crate trait Context: Sized + Clone + Debug + ContextOps<Self> + Aggregate<Self> +
    // these aren't really needed; they are used to make derive happy
    Eq +
    Ord +
    Hash
{
    type Environment: Environment<Self>;
    type GoalInEnvironment: GoalInEnvironment<Self>;
    type CanonicalGoalInEnvironment: CanonicalGoalInEnvironment<Self>;
    type UCanonicalGoalInEnvironment: UCanonicalGoalInEnvironment<Self>;
    type InferenceTable: InferenceTable<Self>;
    type InferenceVariable: InferenceVariable<Self>;
    type UniverseMap: UniverseMap<Self>;
    type Substitution: Substitution<Self>;
    type CanonicalConstrainedSubst: CanonicalConstrainedSubst<Self>;
    type ConstraintInEnvironment: ConstraintInEnvironment<Self>;
    type DomainGoal: DomainGoal<Self>;
    type Goal: Goal<Self>;
    type BindersGoal: BindersGoal<Self>;
    type Parameter: Parameter<Self>;
    type ProgramClause: ProgramClause<Self>;
    type Solution;
}

crate trait ContextOps<C: Context> {
    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &C::UCanonicalGoalInEnvironment) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    fn program_clauses(
        &self,
        environment: &C::Environment,
        goal: &C::DomainGoal,
    ) -> Vec<C::ProgramClause>;

    /// If `subgoal` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_goal(
        &self,
        infer: &mut C::InferenceTable,
        subgoal: &C::GoalInEnvironment,
    ) -> Option<C::GoalInEnvironment>;

    /// If `subst` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_answer(
        &self,
        infer: &mut C::InferenceTable,
        subst: &C::Substitution,
    ) -> Option<C::Substitution>;

    fn resolvent_clause(
        &self,
        infer: &mut C::InferenceTable,
        environment: &C::Environment,
        goal: &C::DomainGoal,
        subst: &C::Substitution,
        clause: &C::ProgramClause,
    ) -> Satisfiable<ExClause<C>>;

    fn apply_answer_subst(
        &self,
        infer: &mut C::InferenceTable,
        ex_clause: ExClause<C>,
        selected_goal: &C::GoalInEnvironment,
        answer_table_goal: &C::CanonicalGoalInEnvironment,
        canonical_answer_subst: &C::CanonicalConstrainedSubst,
    ) -> Satisfiable<ExClause<C>>;

    fn goal_in_environment(
        environment: &C::Environment,
        goal: C::Goal,
    ) -> C::GoalInEnvironment;
}

crate trait Aggregate<C: Context> {
    fn make_solution(
        &self,
        root_goal: &C::CanonicalGoalInEnvironment,
        simplified_answers: impl IntoIterator<Item = SimplifiedAnswer<C>>,
    ) -> Option<C::Solution>;
}

crate trait UCanonicalGoalInEnvironment<C: Context>: Debug + Clone + Eq + Hash {
    fn canonical(&self) -> &C::CanonicalGoalInEnvironment;
    fn is_trivial_substitution(
        &self,
        canonical_subst: &C::CanonicalConstrainedSubst,
    ) -> bool;
}

crate trait CanonicalGoalInEnvironment<C: Context>: Debug + Clone {
    fn substitute(
        &self,
        subst: &C::Substitution,
    ) -> (
        C::Environment,
        C::Goal,
    );
}

crate trait GoalInEnvironment<C: Context>: Debug + Clone + Eq + Ord + Hash {
    fn environment(&self) -> &C::Environment;
}

crate trait Environment<C: Context>: Debug + Clone + Eq + Ord + Hash {
    // Used by: simplify
    fn add_clauses(&self, clauses: impl IntoIterator<Item = C::DomainGoal>) -> Self;
}

crate trait InferenceVariable<C: Context>: Copy {
}

crate trait InferenceTable<C: Context>: Clone {
    type UnificationResult: UnificationResult<C>;

    fn new() -> Self;

    // Used by: simplify
    fn instantiate_binders_universally(
        &mut self,
        arg: &C::BindersGoal,
    ) -> C::Goal;

    // Used by: simplify
    fn instantiate_binders_existentially(
        &mut self,
        arg: &C::BindersGoal,
    ) -> C::Goal;

    // Used by: logic
    fn instantiate_universes<'v>(
        &mut self,
        value: &'v C::UCanonicalGoalInEnvironment,
    ) -> &'v C::CanonicalGoalInEnvironment;

    // Used by: logic (but for debugging only)
    fn debug_ex_clause(&mut self, value: &'v ExClause<C>) -> Box<Debug + 'v>;

    // Used by: logic (but for debugging only)
    fn debug_goal(&mut self, goal: &'v C::GoalInEnvironment) -> Box<Debug + 'v>;

    // Used by: logic
    fn canonicalize_goal(&mut self, value: &C::GoalInEnvironment) -> C::CanonicalGoalInEnvironment;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        subst: C::Substitution,
        constraints: Vec<C::ConstraintInEnvironment>,
    ) -> C::CanonicalConstrainedSubst;

    // Used by: logic
    fn u_canonicalize_goal(
        &mut self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> (C::UCanonicalGoalInEnvironment, C::UniverseMap);

    // Used by: logic
    fn fresh_subst_for_goal(&mut self, goal: &C::CanonicalGoalInEnvironment)
        -> C::Substitution;

    // Used by: logic
    fn invert_goal(&mut self, value: &C::GoalInEnvironment) -> Option<C::GoalInEnvironment>;

    // Used by: simplify
    fn unify_parameters(
        &mut self,
        environment: &C::Environment,
        a: &C::Parameter,
        b: &C::Parameter,
    ) -> Fallible<Self::UnificationResult>;
}

crate trait Substitution<C: Context>: Clone + Debug {
}

crate trait CanonicalConstrainedSubst<C: Context>: Clone + Debug + Eq + Hash + Ord {
    fn empty_constraints(&self) -> bool;
}

crate trait ConstraintInEnvironment<C: Context>: Clone + Debug + Eq + Hash + Ord {
}

crate trait DomainGoal<C: Context>: Clone + Debug + Eq + Hash + Ord {
    fn into_goal(self) -> C::Goal;
}

crate trait Goal<C: Context>: Clone + Debug + Eq + Hash + Ord {
    fn cannot_prove() -> Self;
    fn into_hh_goal(self) -> HhGoal<C>;
}

crate trait Parameter<C: Context>: Clone + Debug + Eq + Hash + Ord {
}

crate trait ProgramClause<C: Context>: Debug {
}

crate trait BindersGoal<C: Context>: Clone + Debug + Eq + Hash + Ord {
}

crate trait UniverseMap<C: Context>: Clone + Debug {
    fn map_goal_from_canonical(
        &self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment;

    fn map_subst_from_canonical(
        &self,
        value: &C::CanonicalConstrainedSubst,
    ) -> C::CanonicalConstrainedSubst;
}

crate trait UnificationResult<C: Context> {
    fn into_ex_clause(self, ex_clause: &mut ExClause<C>);
}
