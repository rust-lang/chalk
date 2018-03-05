use crate::cast::Caster;
use crate::fallible::Fallible;
use crate::ir::*;
use crate::ir::could_match::CouldMatch;
use crate::solve::infer::instantiate::BindersAndValue;
use crate::solve::infer::ucanonicalize::UCanonicalized;
use crate::solve::Solution;
use crate::solve::slg::{CanonicalConstrainedSubst, CanonicalGoal, ExClause, Literal, Satisfiable,
                        UCanonicalGoal};
use crate::solve::slg::context;
use crate::solve::truncate::{self, Truncated};
use crate::fold::Fold;
use std::fmt::Debug;
use std::sync::Arc;

mod resolvent;

#[derive(Clone, Debug)]
pub struct SlgContext {
    program: Arc<ProgramEnvironment<DomainGoal>>,
    max_size: usize,
}

impl SlgContext {
    crate fn new(
        program: &Arc<ProgramEnvironment<DomainGoal>>,
        max_size: usize,
    ) -> SlgContext {
        SlgContext {
            program: program.clone(),
            max_size,
        }
    }

    /// Convenience fn for solving a root goal.
    crate fn solve_root_goal(
        self,
        root_goal: &UCanonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> Option<Solution> {
        use crate::solve::slg::forest::Forest;

        let mut forest = Forest::new(self);
        forest.solve(root_goal)
    }
}

impl context::Context for SlgContext {
    type Environment = Arc<Environment<DomainGoal>>;
    type GoalInEnvironment = InEnvironment<Goal<DomainGoal>>;
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal<DomainGoal>>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal<DomainGoal>>>;
    type InferenceTable = ::crate::solve::infer::InferenceTable;
    type InferenceVariable = ::crate::solve::infer::var::InferenceVariable;
    type UniverseMap = ::crate::solve::infer::ucanonicalize::UniverseMap;

    fn is_coinductive(&self, goal: &UCanonicalGoal<DomainGoal>) -> bool {
        goal.is_coinductive(&self.program)
    }

    fn program_clauses(
        &self,
        environment: &Arc<Environment<DomainGoal>>,
        goal: &DomainGoal,
    ) -> Vec<ProgramClause<DomainGoal>> {
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
        subgoal: &InEnvironment<Goal<DomainGoal>>,
    ) -> Option<InEnvironment<Goal<DomainGoal>>> {
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
        subst: &Substitution,
    ) -> Option<Substitution> {
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
        environment: &Arc<Environment<DomainGoal>>,
        goal: &DomainGoal,
        subst: &Substitution,
        clause: &Binders<ProgramClauseImplication<DomainGoal>>,
    ) -> Satisfiable<ExClause<Self>> {
        resolvent::resolvent_clause(infer, environment, goal, subst, clause)
    }

    fn apply_answer_subst(
        &self,
        infer: &mut Self::InferenceTable,
        ex_clause: ExClause<Self>,
        selected_goal: &InEnvironment<Goal<DomainGoal>>,
        answer_table_goal: &CanonicalGoal<DomainGoal>,
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
        environment: &Arc<Environment<DomainGoal>>,
        goal: Goal<DomainGoal>,
    ) -> InEnvironment<Goal<DomainGoal>> {
        InEnvironment::new(environment, goal)
    }
}

impl context::InferenceTable<SlgContext> for ::crate::solve::infer::InferenceTable {
    type UnificationResult = ::crate::solve::infer::unify::UnificationResult;

    fn new() -> Self {
        Self::new()
    }

    fn fresh_subst(
        &mut self,
        binders: &[ParameterKind<UniverseIndex>],
    ) -> Substitution {
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
        value: &'v UCanonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> &'v Canonical<InEnvironment<Goal<DomainGoal>>> {
        self.instantiate_universes(value)
    }

    fn new_variable(
        &mut self,
        ui: UniverseIndex,
    ) -> ::crate::solve::infer::var::InferenceVariable {
        self.new_variable(ui)
    }

    fn debug_ex_clause(&mut self, value: &'v ExClause<SlgContext>) -> Box<Debug + 'v> {
        Box::new(self.normalize_deep(value))
    }

    fn debug_goal(
        &mut self,
        value: &'v InEnvironment<Goal<DomainGoal>>,
    ) -> Box<Debug + 'v> {
        Box::new(self.normalize_deep(value))
    }

    fn canonicalize_goal(
        &mut self,
        value: &InEnvironment<Goal<DomainGoal>>,
    ) -> Canonical<InEnvironment<Goal<DomainGoal>>> {
        self.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        value: &ConstrainedSubst,
    ) -> Canonical<ConstrainedSubst> {
        self.canonicalize(value).quantified
    }

    fn u_canonicalize_goal(
        &mut self,
        value: &CanonicalGoal<DomainGoal>,
    ) -> (
        UCanonical<InEnvironment<Goal<DomainGoal>>>,
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
        value: &InEnvironment<Goal<DomainGoal>>,
    ) -> Option<InEnvironment<Goal<DomainGoal>>> {
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
        environment: &Arc<Environment<DomainGoal>>,
        a: &Parameter,
        b: &Parameter,
    ) -> Fallible<Self::UnificationResult> {
        self.unify(environment, a, b)
    }
}

impl context::UnificationResult<SlgContext> for ::crate::solve::infer::unify::UnificationResult {
    fn into_ex_clause(self, ex_clause: &mut ExClause<SlgContext>) {
        ex_clause
            .subgoals
            .extend(self.goals.into_iter().casted().map(Literal::Positive));
        ex_clause.constraints.extend(self.constraints);
    }
}

impl context::InferenceVariable<SlgContext> for ::crate::solve::infer::var::InferenceVariable {
    fn to_ty(self) -> Ty {
        self.to_ty()
    }

    fn to_lifetime(self) -> Lifetime {
        self.to_lifetime()
    }
}

impl context::GoalInEnvironment<SlgContext> for InEnvironment<Goal<DomainGoal>> {
    fn environment(&self) -> &Arc<Environment<DomainGoal>> {
        &self.environment
    }
}

impl context::Environment<SlgContext> for Arc<Environment<DomainGoal>> {
    fn add_clauses(&self, clauses: impl IntoIterator<Item = DomainGoal>) -> Self {
        Environment::add_clauses(self, clauses)
    }
}

impl context::UniverseMap<SlgContext> for ::crate::solve::infer::ucanonicalize::UniverseMap {
    fn map_goal_from_canonical(
        &self,
        value: &Canonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> Canonical<InEnvironment<Goal<DomainGoal>>> {
        self.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        &self,
        value: &CanonicalConstrainedSubst,
    ) -> CanonicalConstrainedSubst {
        self.map_from_canonical(value)
    }
}

impl context::CanonicalGoalInEnvironment<SlgContext>
    for Canonical<InEnvironment<Goal<DomainGoal>>>
{
    fn binders(&self) -> &[ParameterKind<UniverseIndex>] {
        &self.binders
    }

    fn substitute(
        &self,
        subst: &Substitution,
    ) -> (
        Arc<Environment<DomainGoal>>,
        Goal<DomainGoal>,
    ) {
        let InEnvironment { environment, goal } = self.substitute(subst);
        (environment, goal)
    }
}

impl context::UCanonicalGoalInEnvironment<SlgContext>
    for UCanonical<InEnvironment<Goal<DomainGoal>>>
{
    fn canonical(&self) -> &Canonical<InEnvironment<Goal<DomainGoal>>> {
        &self.canonical
    }

    fn is_trivial_substitution(
        &self,
        canonical_subst: &Canonical<ConstrainedSubst>,
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
