use crate::cast::{Cast, Caster};
use crate::fallible::Fallible;
use crate::ir::could_match::CouldMatch;
use crate::ir::*;
use crate::solve::infer::ucanonicalize::{UCanonicalized, UniverseMap};
use crate::solve::infer::unify::UnificationResult;
use crate::solve::infer::InferenceTable;
use crate::solve::truncate::{self, Truncated};
use crate::solve::Solution;

use chalk_engine::context;
use chalk_engine::forest::Forest;
use chalk_engine::hh::HhGoal;
use chalk_engine::{DelayedLiteral, ExClause, Literal};

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
    program: Arc<ProgramEnvironment>,
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
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal>>;
    type CanonicalExClause = Canonical<ExClause<Self, Self>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal>>;
    type UniverseMap = UniverseMap;
    type CanonicalConstrainedSubst = Canonical<ConstrainedSubst>;
    type InferenceNormalizedSubst = Substitution;
    type Solution = Solution;
}

impl context::ContextOps<SlgContext> for SlgContext {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal>>) -> bool {
        goal.is_coinductive(&self.program)
    }

    fn instantiate_ucanonical_goal<R>(
        &self,
        arg: &UCanonical<InEnvironment<Goal>>,
        op: impl context::WithInstantiatedUCanonicalGoal<Self, Output = R>,
    ) -> R {
        let (infer, subst, InEnvironment { environment, goal }) =
            InferenceTable::from_canonical(arg.universes, &arg.canonical);
        let dyn_infer = &mut TruncatingInferenceTable::new(&self.program, self.max_size, infer);
        op.with(dyn_infer, subst, environment, goal)
    }

    fn instantiate_ex_clause<R>(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<SlgContext, SlgContext>>,
        op: impl context::WithInstantiatedExClause<Self, Output = R>,
    ) -> R {
        let (infer, _subst, ex_cluse) =
            InferenceTable::from_canonical(num_universes, canonical_ex_clause);
        let dyn_infer = &mut TruncatingInferenceTable::new(&self.program, self.max_size, infer);
        op.with(dyn_infer, ex_cluse)
    }

    fn inference_normalized_subst_from_ex_clause(
        canon_ex_clause: &Canonical<ExClause<SlgContext, SlgContext>>,
    ) -> &Substitution {
        &canon_ex_clause.value.subst
    }

    fn empty_constraints(ccs: &Canonical<ConstrainedSubst>) -> bool {
        ccs.value.constraints.is_empty()
    }

    fn inference_normalized_subst_from_subst(ccs: &Canonical<ConstrainedSubst>) -> &Substitution {
        &ccs.value.subst
    }

    fn canonical(u_canon: &UCanonical<InEnvironment<Goal>>) -> &Canonical<InEnvironment<Goal>> {
        &u_canon.canonical
    }

    fn is_trivial_substitution(u_canon: &UCanonical<InEnvironment<Goal>>,
                               canonical_subst: &Canonical<ConstrainedSubst>) -> bool {
        u_canon.is_trivial_substitution(canonical_subst)
    }

    fn num_universes(u_canon: &UCanonical<InEnvironment<Goal>>) -> usize {
        u_canon.universes
    }

    fn map_goal_from_canonical(
        map: &UniverseMap,
        value: &Canonical<InEnvironment<Goal>>,
    ) -> Canonical<InEnvironment<Goal>> {
        map.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        map: &UniverseMap,
        value: &Canonical<ConstrainedSubst>,
    ) -> Canonical<ConstrainedSubst> {
        map.map_from_canonical(value)
    }
}

impl TruncatingInferenceTable {
    fn new(program: &Arc<ProgramEnvironment>, max_size: usize, infer: InferenceTable) -> Self {
        Self {
            program: program.clone(),
            max_size,
            infer,
        }
    }
}

impl context::TruncateOps<SlgContext, SlgContext> for TruncatingInferenceTable {
    fn truncate_goal(&mut self, subgoal: &InEnvironment<Goal>) -> Option<InEnvironment<Goal>> {
        let Truncated { overflow, value } =
            truncate::truncate(&mut self.infer, self.max_size, subgoal);
        if overflow {
            Some(value)
        } else {
            None
        }
    }

    fn truncate_answer(&mut self, subst: &Substitution) -> Option<Substitution> {
        let Truncated { overflow, value } =
            truncate::truncate(&mut self.infer, self.max_size, subst);
        if overflow {
            Some(value)
        } else {
            None
        }
    }
}

impl context::InferenceTable<SlgContext, SlgContext> for TruncatingInferenceTable {}

impl context::ExClauseContext<SlgContext> for SlgContext {
    type GoalInEnvironment = InEnvironment<Goal>;
    type Substitution = Substitution;
    type RegionConstraint = InEnvironment<Constraint>;
}

impl context::InferenceContext<SlgContext> for SlgContext {
    type Environment = Arc<Environment>;
    type DomainGoal = DomainGoal;
    type Goal = Goal;
    type BindersGoal = Binders<Box<Goal>>;
    type Parameter = Parameter;
    type ProgramClause = ProgramClause;
    type UnificationResult = UnificationResult;

    fn goal_in_environment(environment: &Arc<Environment>, goal: Goal) -> InEnvironment<Goal> {
        InEnvironment::new(environment, goal)
    }

    fn into_goal(domain_goal: Self::DomainGoal) -> Self::Goal {
        domain_goal.cast()
    }

    fn cannot_prove() -> Self::Goal {
        Goal::CannotProve(())
    }

    fn into_hh_goal(goal: Self::Goal) -> HhGoal<SlgContext, Self> {
        match goal {
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

    fn into_ex_clause(
        result: Self::UnificationResult,
        ex_clause: &mut ExClause<SlgContext, SlgContext>,
    ) {
        ex_clause
            .subgoals
            .extend(result.goals.into_iter().casted().map(Literal::Positive));
        ex_clause.constraints.extend(result.constraints);
    }

    // Used by: simplify
    fn add_clauses(
        env: &Self::Environment,
        clauses: impl IntoIterator<Item = Self::ProgramClause>,
    ) -> Self::Environment {
        Environment::add_clauses(env, clauses)
    }
}

impl context::UnificationOps<SlgContext, SlgContext> for TruncatingInferenceTable {
    fn program_clauses(
        &self,
        environment: &Arc<Environment>,
        goal: &DomainGoal,
    ) -> Vec<ProgramClause> {
        let environment_clauses = environment
            .clauses
            .iter()
            .filter(|&env_clause| env_clause.could_match(goal))
            .cloned();

        let program_clauses = self.program
            .program_clauses
            .iter()
            .filter(|&clause| clause.could_match(goal))
            .cloned();

        environment_clauses.chain(program_clauses).collect()
    }

    fn instantiate_binders_universally(&mut self, arg: &Binders<Box<Goal>>) -> Goal {
        *self.infer.instantiate_binders_universally(arg)
    }

    fn instantiate_binders_existentially(&mut self, arg: &Binders<Box<Goal>>) -> Goal {
        *self.infer.instantiate_binders_existentially(arg)
    }

    fn debug_ex_clause(
        &mut self,
        value: &'v ExClause<SlgContext, SlgContext>,
    ) -> Box<dyn Debug + 'v> {
        Box::new(self.infer.normalize_deep(value))
    }

    fn canonicalize_goal(&mut self, value: &InEnvironment<Goal>) -> Canonical<InEnvironment<Goal>> {
        self.infer.canonicalize(value).quantified
    }

    fn canonicalize_ex_clause(
        &mut self,
        value: &ExClause<SlgContext, SlgContext>,
    ) -> Canonical<ExClause<SlgContext, SlgContext>> {
        self.infer.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        subst: Substitution,
        constraints: Vec<InEnvironment<Constraint>>,
    ) -> Canonical<ConstrainedSubst> {
        self.infer
            .canonicalize(&ConstrainedSubst { subst, constraints })
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

impl Substitution {
    fn may_invalidate(&self, subst: &Canonical<Substitution>) -> bool {
        self.parameters
            .iter()
            .zip(&subst.value.parameters)
            .any(|(new, current)| MayInvalidate.aggregate_parameters(new, current))
    }
}

// This is a struct in case we need to add state at any point like in AntiUnifier
struct MayInvalidate;

impl MayInvalidate {
    fn aggregate_parameters(&mut self, new: &Parameter, current: &Parameter) -> bool {
        match (new, current) {
            (ParameterKind::Ty(ty1), ParameterKind::Ty(ty2)) => self.aggregate_tys(ty1, ty2),
            (ParameterKind::Lifetime(l1), ParameterKind::Lifetime(l2)) => {
                self.aggregate_lifetimes(l1, l2)
            }
            (ParameterKind::Ty(_), _) | (ParameterKind::Lifetime(_), _) => panic!(
                "mismatched parameter kinds: new={:?} current={:?}",
                new, current
            ),
        }
    }

    // Returns true if the two types could be unequal.
    fn aggregate_tys(&mut self, new: &Ty, current: &Ty) -> bool {
        match (new, current) {
            (_, Ty::Var(_)) => {
                // If the aggregate solution already has an inference variable here, then no matter
                // what type we produce, the aggregate cannot get 'more generalized' than it already
                // is. So return false, we cannot invalidate.
                false
            }

            (Ty::Var(_), _) => {
                // If we see a type variable in the potential future solution, we have to be
                // conservative. We don't know what type variable will wind up being! Remember that
                // the future solution could be any instantiation of `ty0` -- or it could leave this
                // variable unbound, if the result is true for all types.
                true
            }

            // Aggregating universally-quantified types seems hard according to Niko. ;)
            // Since this is the case, we are conservative here and just say we may invalidate.
            (Ty::ForAll(_), Ty::ForAll(_)) => true,

            (Ty::Apply(apply1), Ty::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (Ty::Projection(apply1), Ty::Projection(apply2)) => {
                self.aggregate_projection_tys(apply1, apply2)
            }

            (Ty::UnselectedProjection(apply1), Ty::UnselectedProjection(apply2)) => {
                self.aggregate_unselected_projection_tys(apply1, apply2)
            }

            (Ty::ForAll(_), _)
            | (Ty::Apply(_), _)
            | (Ty::Projection(_), _)
            | (Ty::UnselectedProjection(_), _) => true,
        }
    }

    fn aggregate_lifetimes(&mut self, _: &Lifetime, _: &Lifetime) -> bool {
        true
    }

    fn aggregate_application_tys(&mut self, new: &ApplicationTy, current: &ApplicationTy) -> bool {
        let ApplicationTy {
            name: new_name,
            parameters: new_parameters,
        } = new;
        let ApplicationTy {
            name: current_name,
            parameters: current_parameters,
        } = current;

        self.aggregate_name_and_substs(new_name, new_parameters, current_name, current_parameters)
    }

    fn aggregate_projection_tys(&mut self, new: &ProjectionTy, current: &ProjectionTy) -> bool {
        let ProjectionTy {
            associated_ty_id: new_name,
            parameters: new_parameters,
        } = new;
        let ProjectionTy {
            associated_ty_id: current_name,
            parameters: current_parameters,
        } = current;

        self.aggregate_name_and_substs(new_name, new_parameters, current_name, current_parameters)
    }

    fn aggregate_unselected_projection_tys(
        &mut self,
        new: &UnselectedProjectionTy,
        current: &UnselectedProjectionTy,
    ) -> bool {
        let UnselectedProjectionTy {
            type_name: new_name,
            parameters: new_parameters,
        } = new;
        let UnselectedProjectionTy {
            type_name: current_name,
            parameters: current_parameters,
        } = current;

        self.aggregate_name_and_substs(new_name, new_parameters, current_name, current_parameters)
    }

    fn aggregate_name_and_substs<N>(
        &mut self,
        new_name: N,
        new_parameters: &[Parameter],
        current_name: N,
        current_parameters: &[Parameter],
    ) -> bool
    where
        N: Copy + Eq + Debug,
    {
        if new_name != current_name {
            return true;
        }

        let name = new_name;

        assert_eq!(
            new_parameters.len(),
            current_parameters.len(),
            "does {:?} take {} parameters or {}? can't both be right",
            name,
            new_parameters.len(),
            current_parameters.len()
        );

        new_parameters
            .iter()
            .zip(current_parameters)
            .any(|(new, current)| self.aggregate_parameters(new, current))
    }
}

type ExClauseSlgContext = ExClause<SlgContext, SlgContext>;
struct_fold!(ExClauseSlgContext {
    subst,
    delayed_literals,
    constraints,
    subgoals,
});

type LiteralSlgContext = Literal<SlgContext, SlgContext>;
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
