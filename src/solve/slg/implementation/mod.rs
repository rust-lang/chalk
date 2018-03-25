use crate::cast::{Cast, Caster};
use crate::fallible::Fallible;
use crate::ir::*;
use crate::ir::could_match::CouldMatch;
use crate::solve::infer::InferenceTable;
use crate::solve::infer::ucanonicalize::{UCanonicalized, UniverseMap};
use crate::solve::infer::unify::UnificationResult;
use crate::solve::Solution;
use crate::solve::truncate::{self, Truncated};

use chalk_engine::{DelayedLiteral, ExClause, Literal};
use chalk_engine::context;
use chalk_engine::forest::Forest;
use chalk_engine::hh::HhGoal;

use std::fmt::Debug;
use std::sync::Arc;

mod aggregate;
mod resolvent;

#[derive(Clone, Debug)]
pub struct SlgContext {
    program: Arc<ProgramEnvironment>,
    max_size: usize,
}

pub struct TruncatingInferenceTable {
    max_size: usize,
    infer: InferenceTable,
}

impl SlgContext {
    crate fn new(program: &Arc<ProgramEnvironment>, max_size: usize) -> SlgContext {
        SlgContext {
            program: program.clone(),
            max_size,
        }
    }

    /// Convenience fn for solving a root goal.
    crate fn solve_root_goal(
        self,
        root_goal: &UCanonical<InEnvironment<Goal>>,
    ) -> Option<Solution> {
        let mut forest = Forest::new(self);
        forest.solve(root_goal)
    }
}

impl context::Context for SlgContext {
    type Environment = Arc<Environment>;
    type GoalInEnvironment = InEnvironment<Goal>;
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal>>;
    type CanonicalExClause = Canonical<ExClause<Self>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal>>;
    type UniverseMap = UniverseMap;
    type Substitution = Substitution;
    type CanonicalConstrainedSubst = Canonical<ConstrainedSubst>;
    type RegionConstraint = InEnvironment<Constraint>;
    type DomainGoal = DomainGoal;
    type Goal = Goal;
    type BindersGoal = Binders<Box<Goal>>;
    type Parameter = Parameter;
    type ProgramClause = ProgramClause;
    type Solution = Solution;
    type UnificationResult = UnificationResult;
}

impl context::ContextOps<SlgContext> for SlgContext {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal>>) -> bool {
        goal.is_coinductive(&self.program)
    }

    fn program_clauses(
        &self,
        environment: &Arc<Environment>,
        goal: &DomainGoal,
    ) -> Vec<ProgramClause> {
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

    fn goal_in_environment(environment: &Arc<Environment>, goal: Goal) -> InEnvironment<Goal> {
        InEnvironment::new(environment, goal)
    }

    fn instantiate_ucanonical_goal<R>(
        &self,
        arg: &UCanonical<InEnvironment<Goal>>,
        op: impl context::WithInstantiatedUCanonicalGoal<Self, Output = R>,
    ) -> R {
        let (infer, subst, InEnvironment { environment, goal }) =
            InferenceTable::from_canonical(arg.universes, &arg.canonical);
        let dyn_infer = &mut TruncatingInferenceTable::new(self.max_size, infer);
        op.with(dyn_infer, subst, environment, goal)
    }

    fn instantiate_ex_clause<R>(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<SlgContext>>,
        op: impl context::WithInstantiatedExClause<Self, Output = R>,
    ) -> R {
        let (infer, _subst, ex_cluse) =
            InferenceTable::from_canonical(num_universes, canonical_ex_clause);
        let dyn_infer = &mut TruncatingInferenceTable::new(self.max_size, infer);
        op.with(dyn_infer, ex_cluse)
    }
}

impl TruncatingInferenceTable {
    fn new(max_size: usize, infer: InferenceTable) -> Self {
        Self { max_size, infer }
    }
}

impl context::TruncateOps<SlgContext> for TruncatingInferenceTable {
    fn truncate_goal(
        &mut self,
        subgoal: &InEnvironment<Goal>,
    ) -> Option<InEnvironment<Goal>> {
        let Truncated { overflow, value } = truncate::truncate(&mut self.infer, self.max_size, subgoal);
        if overflow {
            Some(value)
        } else {
            None
        }
    }

    fn truncate_answer(
        &mut self,
        subst: &Substitution,
    ) -> Option<Substitution> {
        let Truncated { overflow, value } = truncate::truncate(&mut self.infer, self.max_size, subst);
        if overflow {
            Some(value)
        } else {
            None
        }
    }
}

impl context::InferenceTable<SlgContext> for TruncatingInferenceTable {
    fn instantiate_binders_universally(&mut self, arg: &Binders<Box<Goal>>) -> Goal {
        *self.infer.instantiate_binders_universally(arg)
    }

    fn instantiate_binders_existentially(&mut self, arg: &Binders<Box<Goal>>) -> Goal {
        *self.infer.instantiate_binders_existentially(arg)
    }

    fn debug_ex_clause(&mut self, value: &'v ExClause<SlgContext>) -> Box<dyn Debug + 'v> {
        Box::new(self.infer.normalize_deep(value))
    }

    fn canonicalize_goal(&mut self, value: &InEnvironment<Goal>) -> Canonical<InEnvironment<Goal>> {
        self.infer.canonicalize(value).quantified
    }

    fn canonicalize_ex_clause(&mut self, value: &ExClause<SlgContext>) -> Canonical<ExClause<SlgContext>> {
        self.infer.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        subst: Substitution,
        constraints: Vec<InEnvironment<Constraint>>,
    ) -> Canonical<ConstrainedSubst> {
        self.infer.canonicalize(&ConstrainedSubst { subst, constraints })
            .quantified
    }

    fn u_canonicalize_goal(
        &mut self,
        value: &Canonical<InEnvironment<Goal>>,
    ) -> (
        UCanonical<InEnvironment<Goal>>,
        ::crate::solve::infer::ucanonicalize::UniverseMap,
    ) {
        let UCanonicalized {
            quantified,
            universes,
        } = self.infer.u_canonicalize(value);
        (quantified, universes)
    }

    fn invert_goal(&mut self, value: &InEnvironment<Goal>) -> Option<InEnvironment<Goal>> {
        self.infer.invert(value)
    }

    fn unify_parameters(
        &mut self,
        environment: &Arc<Environment>,
        a: &Parameter,
        b: &Parameter,
    ) -> Fallible<UnificationResult> {
        self.infer.unify(environment, a, b)
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

impl context::GoalInEnvironment<SlgContext> for InEnvironment<Goal> {
    fn environment(&self) -> &Arc<Environment> {
        &self.environment
    }
}

impl context::Environment<SlgContext> for Arc<Environment> {
    fn add_clauses(&self, clauses: impl IntoIterator<Item = DomainGoal>) -> Self {
        Environment::add_clauses(self, clauses)
    }
}

impl context::UniverseMap<SlgContext> for ::crate::solve::infer::ucanonicalize::UniverseMap {
    fn map_goal_from_canonical(
        &self,
        value: &Canonical<InEnvironment<Goal>>,
    ) -> Canonical<InEnvironment<Goal>> {
        self.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        &self,
        value: &Canonical<ConstrainedSubst>,
    ) -> Canonical<ConstrainedSubst> {
        self.map_from_canonical(value)
    }
}

impl context::DomainGoal<SlgContext> for DomainGoal {
    fn into_goal(self) -> Goal {
        self.cast()
    }
}

impl context::CanonicalConstrainedSubst<SlgContext> for Canonical<ConstrainedSubst> {
    fn empty_constraints(&self) -> bool {
        self.value.constraints.is_empty()
    }
}

impl context::UCanonicalGoalInEnvironment<SlgContext> for UCanonical<InEnvironment<Goal>> {
    fn num_universes(&self) -> usize {
        self.universes
    }

    fn canonical(&self) -> &Canonical<InEnvironment<Goal>> {
        &self.canonical
    }

    fn is_trivial_substitution(&self, canonical_subst: &Canonical<ConstrainedSubst>) -> bool {
        self.is_trivial_substitution(canonical_subst)
    }
}

impl context::Goal<SlgContext> for Goal {
    fn cannot_prove() -> Goal {
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

copy_fold!(::chalk_engine::TableIndex);

type DelayedLiteralSlgContext = DelayedLiteral<SlgContext>;
enum_fold!(DelayedLiteralSlgContext {
    DelayedLiteral :: {
        CannotProve(a), Negative(a), Positive(a, b)
    }
});
