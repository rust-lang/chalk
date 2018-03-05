use crate::cast::Caster;
use crate::fallible::Fallible;
use crate::ir;
use crate::ir::could_match::CouldMatch;
use crate::solve::infer::instantiate::BindersAndValue;
use crate::solve::infer::ucanonicalize::UCanonicalized;
use crate::solve::slg::{CanonicalConstrainedSubst, CanonicalGoal, ExClause, Literal, Satisfiable,
                        UCanonicalGoal};
use crate::solve::slg::context::prelude::*;
use crate::solve::truncate::{self, Truncated};
use crate::fold::Fold;
use std::fmt::Debug;
use std::sync::Arc;

mod resolvent;

#[derive(Clone, Debug)]
pub struct SlgContext {
    program: Arc<ir::ProgramEnvironment<ir::DomainGoal>>,
    max_size: usize,
}

impl SlgContext {
    crate fn new(
        program: &Arc<ir::ProgramEnvironment<ir::DomainGoal>>,
        max_size: usize,
    ) -> SlgContext {
        SlgContext {
            program: program.clone(),
            max_size,
        }
    }
}

impl Context for SlgContext {
    type Environment = Arc<ir::Environment<ir::DomainGoal>>;
    type GoalInEnvironment = ir::InEnvironment<ir::Goal<ir::DomainGoal>>;
    type CanonicalGoalInEnvironment = ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>;
    type UCanonicalGoalInEnvironment = ir::UCanonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>;
    type InferenceTable = ::crate::solve::infer::InferenceTable;
    type InferenceVariable = ::crate::solve::infer::var::InferenceVariable;
    type UniverseMap = ::crate::solve::infer::ucanonicalize::UniverseMap;

    fn is_coinductive(&self, goal: &UCanonicalGoal<ir::DomainGoal>) -> bool {
        goal.is_coinductive(&self.program)
    }

    fn program_clauses(
        &self,
        environment: &Arc<ir::Environment<ir::DomainGoal>>,
        goal: &ir::DomainGoal,
    ) -> Vec<ir::ProgramClause<ir::DomainGoal>> {
        let environment_clauses = environment
            .clauses
            .iter()
            .filter(|&env_clause| env_clause.could_match(goal))
            .map(|env_clause| env_clause.clone().into_program_clause());

        let program_clauses = self.program
            .program_clauses
            .iter()
            .filter(|clause| clause.could_match(goal))
            .cloned();

        environment_clauses.chain(program_clauses).collect()
    }

    /// If `subgoal` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_goal(
        &self,
        infer: &mut Self::InferenceTable,
        subgoal: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> Option<ir::InEnvironment<ir::Goal<ir::DomainGoal>>> {
        let Truncated { overflow, value } = truncate::truncate(infer, self.max_size, subgoal);
        if overflow {
            Some(value)
        } else {
            None
        }
    }

    /// If `subst` is too large, return a truncated variant (else
    /// return `None`).
    fn truncate_answer(
        &self,
        infer: &mut Self::InferenceTable,
        subst: &ir::Substitution,
    ) -> Option<ir::Substitution> {
        let Truncated { overflow, value } = truncate::truncate(infer, self.max_size, subst);
        if overflow {
            Some(value)
        } else {
            None
        }
    }

    fn resolvent_clause(
        &self,
        infer: &mut Self::InferenceTable,
        environment: &Arc<ir::Environment<ir::DomainGoal>>,
        goal: &ir::DomainGoal,
        subst: &ir::Substitution,
        clause: &ir::Binders<ir::ProgramClauseImplication<ir::DomainGoal>>,
    ) -> Satisfiable<ExClause<Self>> {
        resolvent::resolvent_clause(infer, environment, goal, subst, clause)
    }

    fn apply_answer_subst(
        &self,
        infer: &mut Self::InferenceTable,
        ex_clause: ExClause<Self>,
        selected_goal: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
        answer_table_goal: &CanonicalGoal<ir::DomainGoal>,
        canonical_answer_subst: &CanonicalConstrainedSubst,
    ) -> Satisfiable<ExClause<Self>> {
        resolvent::apply_answer_subst(
            infer,
            ex_clause,
            selected_goal,
            answer_table_goal,
            canonical_answer_subst,
        )
    }

    fn goal_in_environment(
        environment: &Arc<ir::Environment<ir::DomainGoal>>,
        goal: ir::Goal<ir::DomainGoal>,
    ) -> ir::InEnvironment<ir::Goal<ir::DomainGoal>> {
        ir::InEnvironment::new(environment, goal)
    }
}

impl InferenceTable<SlgContext> for ::crate::solve::infer::InferenceTable {
    type UnificationResult = ::crate::solve::infer::unify::UnificationResult;

    fn new() -> Self {
        Self::new()
    }

    fn fresh_subst(
        &mut self,
        binders: &[ir::ParameterKind<ir::UniverseIndex>],
    ) -> ir::Substitution {
        self.fresh_subst(binders)
    }

    fn instantiate_binders_universally<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold,
    {
        self.instantiate_binders_universally(arg)
    }

    fn instantiate_universes<'v>(
        &mut self,
        value: &'v ir::UCanonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>,
    ) -> &'v ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>> {
        self.instantiate_universes(value)
    }

    fn new_variable(
        &mut self,
        ui: ir::UniverseIndex,
    ) -> ::crate::solve::infer::var::InferenceVariable {
        self.new_variable(ui)
    }

    fn debug_ex_clause(&mut self, value: &'v ExClause<SlgContext>) -> Box<Debug + 'v> {
        Box::new(self.normalize_deep(value))
    }

    fn debug_goal(
        &mut self,
        value: &'v ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> Box<Debug + 'v> {
        Box::new(self.normalize_deep(value))
    }

    fn canonicalize_goal(
        &mut self,
        value: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>> {
        self.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        value: &ir::ConstrainedSubst,
    ) -> ir::Canonical<ir::ConstrainedSubst> {
        self.canonicalize(value).quantified
    }

    fn u_canonicalize_goal(
        &mut self,
        value: &CanonicalGoal<ir::DomainGoal>,
    ) -> (
        ir::UCanonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>,
        ::crate::solve::infer::ucanonicalize::UniverseMap,
    ) {
        let UCanonicalized {
            quantified,
            universes,
        } = self.u_canonicalize(value);
        (quantified, universes)
    }

    fn invert_goal(
        &mut self,
        value: &ir::InEnvironment<ir::Goal<ir::DomainGoal>>,
    ) -> Option<ir::InEnvironment<ir::Goal<ir::DomainGoal>>> {
        self.invert(value)
    }

    fn instantiate_binders_existentially<T>(
        &mut self,
        arg: &impl BindersAndValue<Output = T>,
    ) -> T::Result
    where
        T: Fold,
    {
        self.instantiate_binders_existentially(arg)
    }

    fn unify_parameters(
        &mut self,
        environment: &Arc<ir::Environment<ir::DomainGoal>>,
        a: &ir::Parameter,
        b: &ir::Parameter,
    ) -> Fallible<Self::UnificationResult> {
        self.unify(environment, a, b)
    }
}

impl UnificationResult<SlgContext> for ::crate::solve::infer::unify::UnificationResult {
    fn into_ex_clause(self, ex_clause: &mut ExClause<SlgContext>) {
        ex_clause
            .subgoals
            .extend(self.goals.into_iter().casted().map(Literal::Positive));
        ex_clause.constraints.extend(self.constraints);
    }
}

impl InferenceVariable<SlgContext> for ::crate::solve::infer::var::InferenceVariable {
    fn to_ty(self) -> ir::Ty {
        self.to_ty()
    }

    fn to_lifetime(self) -> ir::Lifetime {
        self.to_lifetime()
    }
}

impl GoalInEnvironment<SlgContext> for ir::InEnvironment<ir::Goal<ir::DomainGoal>> {
    fn environment(&self) -> &Arc<ir::Environment<ir::DomainGoal>> {
        &self.environment
    }
}

impl Environment<SlgContext> for Arc<ir::Environment<ir::DomainGoal>> {
    fn add_clauses(&self, clauses: impl IntoIterator<Item = ir::DomainGoal>) -> Self {
        ir::Environment::add_clauses(self, clauses)
    }
}


impl UniverseMap<SlgContext> for ::crate::solve::infer::ucanonicalize::UniverseMap {
    fn map_goal_from_canonical(
        &self,
        value: &ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>,
    ) -> ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>> {
        self.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        &self,
        value: &CanonicalConstrainedSubst,
    ) -> CanonicalConstrainedSubst {
        self.map_from_canonical(value)
    }
}

impl CanonicalGoalInEnvironment<SlgContext>
    for ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>
{
    fn binders(&self) -> &[ir::ParameterKind<ir::UniverseIndex>] {
        &self.binders
    }

    fn substitute(
        &self,
        subst: &ir::Substitution,
    ) -> (
        Arc<ir::Environment<ir::DomainGoal>>,
        ir::Goal<ir::DomainGoal>,
    ) {
        let ir::InEnvironment { environment, goal } = self.substitute(subst);
        (environment, goal)
    }
}

impl UCanonicalGoalInEnvironment<SlgContext>
    for ir::UCanonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>>
{
    fn canonical(&self) -> &ir::Canonical<ir::InEnvironment<ir::Goal<ir::DomainGoal>>> {
        &self.canonical
    }

    fn is_trivial_substitution(
        &self,
        canonical_subst: &ir::Canonical<ir::ConstrainedSubst>,
    ) -> bool {
        self.is_trivial_substitution(canonical_subst)
    }
}

type ExClauseSlgContext = ExClause<SlgContext>;
struct_fold!(ExClauseSlgContext {
    subst,
    delayed_literals,
    constraints,
    subgoals,
});

type LiteralSlgContext = Literal<SlgContext>;
impl Fold for LiteralSlgContext {
    type Result = LiteralSlgContext;
    fn fold_with(&self, folder: &mut ::fold::Folder, binders: usize) -> Fallible<Self::Result> {
        match self {
            Literal::Positive(goal) => Ok(Literal::Positive(goal.fold_with(folder, binders)?)),
            Literal::Negative(goal) => Ok(Literal::Negative(goal.fold_with(folder, binders)?)),
        }
    }
}
