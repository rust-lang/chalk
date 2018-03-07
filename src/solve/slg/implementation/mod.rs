use crate::cast::{Cast, Caster};
use crate::fallible::Fallible;
use crate::ir::*;
use crate::ir::could_match::CouldMatch;
use crate::solve::infer::InferenceTable;
use crate::solve::infer::ucanonicalize::{UCanonicalized, UniverseMap};
use crate::solve::infer::unify::UnificationResult;
use crate::solve::infer::var::InferenceVariable;
use crate::solve::Solution;
use crate::solve::truncate::{self, Truncated};

use chalk_slg::{DelayedLiteral, ExClause, Literal};
use chalk_slg::context;
use chalk_slg::forest::Forest;
use chalk_slg::hh::HhGoal;

use std::fmt::Debug;
use std::sync::Arc;

mod aggregate;
mod resolvent;

#[derive(Clone, Debug)]
pub struct SlgContext {
    program: Arc<ProgramEnvironment<DomainGoal>>,
    max_size: usize,
}

impl SlgContext {
    crate fn new(program: &Arc<ProgramEnvironment<DomainGoal>>, max_size: usize) -> SlgContext {
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
        let mut forest = Forest::new(self);
        forest.solve(root_goal)
    }
}

impl context::Context for SlgContext {
    type Environment = Arc<Environment<DomainGoal>>;
    type GoalInEnvironment = InEnvironment<Goal<DomainGoal>>;
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal<DomainGoal>>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal<DomainGoal>>>;
    type InferenceTable = InferenceTable;
    type InferenceVariable = InferenceVariable;
    type UniverseMap = UniverseMap;
    type Substitution = Substitution;
    type CanonicalConstrainedSubst = Canonical<ConstrainedSubst>;
    type ConstraintInEnvironment = InEnvironment<Constraint>;
    type DomainGoal = DomainGoal;
    type Goal = Goal<DomainGoal>;
    type BindersGoal = Binders<Box<Goal<DomainGoal>>>;
    type Parameter = Parameter;
    type ProgramClause = ProgramClause<DomainGoal>;
    type Solution = Solution;
}

impl context::ContextOps<SlgContext> for SlgContext {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal<DomainGoal>>>) -> bool {
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
        infer: &mut InferenceTable,
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
        infer: &mut InferenceTable,
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
        infer: &mut InferenceTable,
        environment: &Arc<Environment<DomainGoal>>,
        goal: &DomainGoal,
        subst: &Substitution,
        clause: &ProgramClause<DomainGoal>,
    ) -> Fallible<ExClause<Self>> {
        resolvent::resolvent_clause(infer, environment, goal, subst, &clause.implication)
    }

    fn apply_answer_subst(
        &self,
        infer: &mut InferenceTable,
        ex_clause: ExClause<Self>,
        selected_goal: &InEnvironment<Goal<DomainGoal>>,
        answer_table_goal: &Canonical<InEnvironment<Goal<DomainGoal>>>,
        canonical_answer_subst: &Canonical<ConstrainedSubst>,
    ) -> Fallible<ExClause<Self>> {
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

impl context::InferenceTable<SlgContext> for InferenceTable {
    type UnificationResult = UnificationResult;

    fn new() -> Self {
        Self::new()
    }

    fn fresh_subst_for_goal(
        &mut self,
        goal: &Canonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> Substitution {
        self.fresh_subst(&goal.binders)
    }

    fn instantiate_binders_universally(
        &mut self,
        arg: &Binders<Box<Goal<DomainGoal>>>,
    ) -> Goal<DomainGoal> {
        *self.instantiate_binders_universally(arg)
    }

    fn instantiate_binders_existentially(
        &mut self,
        arg: &Binders<Box<Goal<DomainGoal>>>,
    ) -> Goal<DomainGoal> {
        *self.instantiate_binders_existentially(arg)
    }

    fn instantiate_universes<'v>(
        &mut self,
        value: &'v UCanonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> &'v Canonical<InEnvironment<Goal<DomainGoal>>> {
        self.instantiate_universes(value)
    }

    fn debug_ex_clause(&mut self, value: &'v ExClause<SlgContext>) -> Box<Debug + 'v> {
        Box::new(self.normalize_deep(value))
    }

    fn debug_goal(&mut self, value: &'v InEnvironment<Goal<DomainGoal>>) -> Box<Debug + 'v> {
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
        subst: Substitution,
        constraints: Vec<InEnvironment<Constraint>>,
    ) -> Canonical<ConstrainedSubst> {
        self.canonicalize(&ConstrainedSubst { subst, constraints })
            .quantified
    }

    fn u_canonicalize_goal(
        &mut self,
        value: &Canonical<InEnvironment<Goal<DomainGoal>>>,
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

impl context::InferenceVariable<SlgContext> for ::crate::solve::infer::var::InferenceVariable {}

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

impl context::Substitution<SlgContext> for Substitution {}

impl context::Parameter<SlgContext> for Parameter {}

impl context::UniverseMap<SlgContext> for ::crate::solve::infer::ucanonicalize::UniverseMap {
    fn map_goal_from_canonical(
        &self,
        value: &Canonical<InEnvironment<Goal<DomainGoal>>>,
    ) -> Canonical<InEnvironment<Goal<DomainGoal>>> {
        self.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        &self,
        value: &Canonical<ConstrainedSubst>,
    ) -> Canonical<ConstrainedSubst> {
        self.map_from_canonical(value)
    }
}

impl context::ConstraintInEnvironment<SlgContext> for InEnvironment<Constraint> {}

impl context::DomainGoal<SlgContext> for DomainGoal {
    fn into_goal(self) -> Goal<DomainGoal> {
        self.cast()
    }
}

impl context::CanonicalConstrainedSubst<SlgContext> for Canonical<ConstrainedSubst> {
    fn empty_constraints(&self) -> bool {
        self.value.constraints.is_empty()
    }
}

impl context::CanonicalGoalInEnvironment<SlgContext>
    for Canonical<InEnvironment<Goal<DomainGoal>>>
{
    fn substitute(&self, subst: &Substitution) -> (Arc<Environment<DomainGoal>>, Goal<DomainGoal>) {
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

    fn is_trivial_substitution(&self, canonical_subst: &Canonical<ConstrainedSubst>) -> bool {
        self.is_trivial_substitution(canonical_subst)
    }
}

impl context::BindersGoal<SlgContext> for Binders<Box<Goal<DomainGoal>>> {}

impl context::ProgramClause<SlgContext> for ProgramClause<DomainGoal> {}

impl context::Goal<SlgContext> for Goal<DomainGoal> {
    fn cannot_prove() -> Goal<DomainGoal> {
        Goal::CannotProve(())
    }

    fn into_hh_goal(self) -> HhGoal<SlgContext> {
        match self {
            Goal::Quantified(QuantifierKind::ForAll, binders_goal) => HhGoal::ForAll(binders_goal),
            Goal::Quantified(QuantifierKind::Exists, binders_goal) => HhGoal::Exists(binders_goal),
            Goal::Implies(dg, subgoal) => HhGoal::Implies(dg, *subgoal),
            Goal::And(g1, g2) => HhGoal::And(*g1, *g2),
            Goal::Not(g1) => HhGoal::Not(*g1),
            Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })) => HhGoal::Unify(a, b),
            Goal::Leaf(LeafGoal::DomainGoal(domain_goal)) => HhGoal::DomainGoal(domain_goal),
            Goal::CannotProve(()) => HhGoal::CannotProve,
        }
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
enum_fold!(LiteralSlgContext {
    Literal :: {
        Positive(a), Negative(a)
    }
});

copy_fold!(::chalk_slg::TableIndex);

type DelayedLiteralSlgContext = DelayedLiteral<SlgContext>;
enum_fold!(DelayedLiteralSlgContext {
    DelayedLiteral :: {
        CannotProve(a), Negative(a), Positive(a, b)
    }
});
