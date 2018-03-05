use crate::fallible::Fallible;
use crate::ir;
use crate::solve::infer::instantiate::BindersAndValue;
use crate::solve::infer::ucanonicalize::UCanonicalized;
use crate::solve::slg::{CanonicalConstrainedSubst, CanonicalGoal, ExClause, Satisfiable,
                        UCanonicalGoal};
use crate::fold::Fold;
use std::sync::Arc;

crate mod implementation;

crate trait Context: Sized + Clone {
    type InferenceTable: InferenceTable<Self>;
    type InferenceVariable: InferenceVariable<Self>;

    /// True if this is a coinductive goal -- e.g., proving an auto trait.
    fn is_coinductive(&self, goal: &UCanonicalGoal<ir::DomainGoal>) -> bool;

    /// Returns the set of program clauses that might apply to
    /// `goal`. (This set can be over-approximated, naturally.)
    fn program_clauses(
        &self,
        environment: &ir::Environment<ir::DomainGoal>,
        goal: &ir::DomainGoal,
    ) -> Vec<ir::ProgramClause<ir::DomainGoal>>;

    /// If `subgoal` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_goal(
        &self,
        infer: &mut Self::InferenceTable,
        subgoal: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> Option<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>;

    /// If `subst` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_answer(
        &self,
        infer: &mut Self::InferenceTable,
        subst: &ir::Substitution,
    ) -> Option<ir::Substitution>;

    fn resolvent_clause(
        &self,
        infer: &mut Self::InferenceTable,
        environment: &Arc<ir::Environment<ir::DomainGoal>>,
        goal: &ir::DomainGoal,
        subst: &ir::Substitution,
        clause: &ir::Binders<ir::ProgramClauseImplication<ir::DomainGoal>>,
    ) -> Satisfiable<ExClause>;

    fn apply_answer_subst(
        &self,
        infer: &mut Self::InferenceTable,
        ex_clause: ExClause,
        selected_goal: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
        answer_table_goal: &CanonicalGoal<ir::DomainGoal>,
        canonical_answer_subst: &CanonicalConstrainedSubst,
    ) -> Satisfiable<ExClause>;
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
    fn instantiate_universes<'v, T>(
        &mut self,
        value: &'v ir::UCanonical<T>,
    ) -> &'v ir::Canonical<T>;

    // Used by: aggregate
    fn new_variable(&mut self, ui: ir::UniverseIndex) -> C::InferenceVariable;

    // Used by: logic (but for debugging only)
    fn normalize_deep<T: Fold>(&mut self, value: &T) -> T::Result;

    // Used by: logic
    fn canonicalize_goal(
        &mut self,
        value: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>;

    // Used by: logic
    fn canonicalize_constrained_subst(
        &mut self,
        value: &ir::ConstrainedSubst,
    ) -> ir::Canonical<ir::ConstrainedSubst>;

    // Used by: logic
    fn u_canonicalize_goal(
        &mut self,
        value: &CanonicalGoal<ir::DomainGoal>,
    ) -> UCanonicalized<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>;

    // Used by: logic
    fn fresh_subst(&mut self, binders: &[ir::ParameterKind<ir::UniverseIndex>])
        -> ir::Substitution;

    // Used by: logic
    fn invert_goal(
        &mut self,
        value: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> Option<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>;

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
        environment: &Arc<ir::Environment<ir::DomainGoal>>,
        a: &ir::Parameter,
        b: &ir::Parameter,
    ) -> Fallible<Self::UnificationResult>;
}

crate trait UnificationResult<C: Context> {
    fn into_ex_clause(self, ex_clause: &mut ExClause);
}

crate mod prelude {
    #![allow(unused_imports)] // rustc bug

    crate use super::Context;
    crate use super::InferenceTable;
    crate use super::InferenceVariable;
    crate use super::UnificationResult;
}
