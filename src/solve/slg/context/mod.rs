use crate::fallible::Fallible;
use crate::ir;
use crate::solve::infer::instantiate::BindersAndValue;
use crate::solve::slg::{CanonicalConstrainedSubst, ExClause, Satisfiable};
use crate::fold::Fold;
use std::fmt::Debug;
use std::hash::Hash;

crate mod implementation;
crate mod prelude;

crate trait Context: Sized + Clone + Debug + ContextOps<Self> + Resolvent<Self> {
    type Environment: Environment<Self>;
    type GoalInEnvironment: GoalInEnvironment<Self>;
    type CanonicalGoalInEnvironment: CanonicalGoalInEnvironment<Self>;
    type UCanonicalGoalInEnvironment: UCanonicalGoalInEnvironment<Self>;
    type InferenceTable: InferenceTable<Self>;
    type InferenceVariable: InferenceVariable<Self>;
    type UniverseMap: UniverseMap<Self>;
}

crate trait ContextOps<C: Context> {
    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &C::UCanonicalGoalInEnvironment) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    fn program_clauses(
        &self,
        environment: &C::Environment,
        goal: &ir::DomainGoal,
    ) -> Vec<ir::ProgramClause<ir::DomainGoal>>;

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
        subst: &ir::Substitution,
    ) -> Option<ir::Substitution>;

    fn resolvent_clause(
        &self,
        infer: &mut C::InferenceTable,
        environment: &C::Environment,
        goal: &ir::DomainGoal,
        subst: &ir::Substitution,
        clause: &ir::Binders<ir::ProgramClauseImplication<ir::DomainGoal>>,
    ) -> Satisfiable<ExClause<C>>;

    fn apply_answer_subst(
        &self,
        infer: &mut C::InferenceTable,
        ex_clause: ExClause<C>,
        selected_goal: &C::GoalInEnvironment,
        answer_table_goal: &C::CanonicalGoalInEnvironment,
        canonical_answer_subst: &CanonicalConstrainedSubst,
    ) -> Satisfiable<ExClause<C>>;

    fn goal_in_environment(
        environment: &C::Environment,
        goal: ir::Goal<ir::DomainGoal>,
    ) -> C::GoalInEnvironment;
}

crate trait UCanonicalGoalInEnvironment<C: Context>: Debug + Clone + Eq + Hash {
    fn canonical(&self) -> &C::CanonicalGoalInEnvironment;
    fn is_trivial_substitution(
        &self,
        canonical_subst: &ir::Canonical<ir::ConstrainedSubst>,
    ) -> bool;
}

crate trait CanonicalGoalInEnvironment<C: Context>: Debug + Clone {
    fn binders(&self) -> &[ir::ParameterKind<ir::UniverseIndex>];
    fn substitute(
        &self,
        subst: &ir::Substitution,
    ) -> (
        C::Environment,
        ir::Goal<ir::DomainGoal>,
    );
}

crate trait GoalInEnvironment<C: Context>: Debug + Clone + Eq + Ord + Hash {
    fn environment(&self) -> &C::Environment;
}

crate trait Environment<C: Context>: Debug + Clone + Eq + Ord + Hash {
    // Used by: simplify
    fn add_clauses(&self, clauses: impl IntoIterator<Item = ir::DomainGoal>) -> Self;
}

crate trait InferenceVariable<C: Context>: Copy {
    fn to_ty(self) -> ir::Ty;
    fn to_lifetime(self) -> ir::Lifetime;
}

crate trait InferenceTable<C: Context>: Clone {
    type UnificationResult: UnificationResult<C>;

    fn new() -> Self;

    // Used by: simplify
    fn instantiate_binders_universally<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold;

    // Used by: logic
    fn instantiate_universes<'v>(
        &mut self,
        value: &'v C::UCanonicalGoalInEnvironment,
    ) -> &'v C::CanonicalGoalInEnvironment;

    // Used by: aggregate
    fn new_variable(&mut self, ui: ir::UniverseIndex) -> C::InferenceVariable;

    // Used by: logic (but for debugging only)
    fn debug_ex_clause(&mut self, value: &'v ExClause<C>) -> Box<Debug + 'v>;

    // Used by: logic (but for debugging only)
    fn debug_goal(&mut self, goal: &'v C::GoalInEnvironment) -> Box<Debug + 'v>;

    // Used by: logic
    fn canonicalize_goal(&mut self, value: &C::GoalInEnvironment) -> C::CanonicalGoalInEnvironment;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        value: &ir::ConstrainedSubst,
    ) -> ir::Canonical<ir::ConstrainedSubst>;

    // Used by: logic
    fn u_canonicalize_goal(
        &mut self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> (C::UCanonicalGoalInEnvironment, C::UniverseMap);

    // Used by: logic
    fn fresh_subst(&mut self, binders: &[ir::ParameterKind<ir::UniverseIndex>])
        -> ir::Substitution;

    // Used by: logic
    fn invert_goal(&mut self, value: &C::GoalInEnvironment) -> Option<C::GoalInEnvironment>;

    // Used by: simplify
    fn instantiate_binders_existentially<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold;

    // Used by: simplify
    fn unify_parameters(
        &mut self,
        environment: &C::Environment,
        a: &ir::Parameter,
        b: &ir::Parameter,
    ) -> Fallible<Self::UnificationResult>;
}

crate trait UniverseMap<C: Context>: Clone + Debug {
    fn map_goal_from_canonical(
        &self,
        value: &C::CanonicalGoalInEnvironment,
    ) -> C::CanonicalGoalInEnvironment;

    fn map_subst_from_canonical(
        &self,
        value: &CanonicalConstrainedSubst,
    ) -> CanonicalConstrainedSubst;
}

crate trait UnificationResult<C: Context> {
    fn into_ex_clause(self, ex_clause: &mut ExClause<C>);
}
