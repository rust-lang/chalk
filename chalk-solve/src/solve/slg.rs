use crate::infer::ucanonicalize::{UCanonicalized, UniverseMap};
use crate::infer::unify::UnificationResult;
use crate::infer::InferenceTable;
use crate::solve::truncate::{self, Truncated};
use crate::solve::ProgramClauseSet;
use crate::solve::Solution;
use chalk_engine::fallible::Fallible;
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::*;

use chalk_engine::context;
use chalk_engine::hh::HhGoal;
use chalk_engine::{DelayedLiteral, ExClause, Literal};

use std::fmt::Debug;
use std::sync::Arc;

mod aggregate;
mod resolvent;

#[derive(Clone, Debug)]
pub(crate) struct SlgContext {
    max_size: usize,
}

impl SlgContext {
    pub(crate) fn new(max_size: usize) -> SlgContext {
        SlgContext { max_size }
    }

    pub(crate) fn ops<'p>(&self, program: &'p dyn ProgramClauseSet) -> SlgContextOps<'p> {
        SlgContextOps {
            program,
            max_size: self.max_size,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SlgContextOps<'me> {
    program: &'me dyn ProgramClauseSet,
    max_size: usize,
}

pub(super) struct TruncatingInferenceTable<'me> {
    program: &'me dyn ProgramClauseSet,
    max_size: usize,
    infer: InferenceTable,
}

impl context::Context for SlgContext {
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal>>;
    type CanonicalExClause = Canonical<ExClause<Self>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal>>;
    type UniverseMap = UniverseMap;
    type InferenceNormalizedSubst = Substitution;
    type Solution = Solution;
    type Environment = Arc<Environment>;
    type DomainGoal = DomainGoal;
    type Goal = Goal;
    type BindersGoal = Binders<Box<Goal>>;
    type Parameter = Parameter;
    type ProgramClause = ProgramClause;
    type ProgramClauses = Vec<ProgramClause>;
    type UnificationResult = UnificationResult;
    type CanonicalConstrainedSubst = Canonical<ConstrainedSubst>;
    type GoalInEnvironment = InEnvironment<Goal>;
    type Substitution = Substitution;
    type RegionConstraint = InEnvironment<Constraint>;
    type Variance = ();

    fn goal_in_environment(environment: &Arc<Environment>, goal: Goal) -> InEnvironment<Goal> {
        InEnvironment::new(environment, goal)
    }

    fn inference_normalized_subst_from_ex_clause(
        canon_ex_clause: &Canonical<ExClause<SlgContext>>,
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

    fn is_trivial_substitution(
        u_canon: &UCanonical<InEnvironment<Goal>>,
        canonical_subst: &Canonical<ConstrainedSubst>,
    ) -> bool {
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

impl<'me> context::ContextOps<SlgContext> for SlgContextOps<'me> {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal>>) -> bool {
        goal.is_coinductive(IsCoinductive::as_dyn(self.program))
    }

    fn instantiate_ucanonical_goal<R>(
        &self,
        arg: &UCanonical<InEnvironment<Goal>>,
        op: impl context::WithInstantiatedUCanonicalGoal<SlgContext, Output = R>,
    ) -> R {
        let (infer, subst, InEnvironment { environment, goal }) =
            InferenceTable::from_canonical(arg.universes, &arg.canonical);
        let dyn_infer = &mut TruncatingInferenceTable::new(self.program, self.max_size, infer);
        op.with(dyn_infer, subst, environment, goal)
    }

    fn instantiate_ex_clause<R>(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<SlgContext>>,
        op: impl context::WithInstantiatedExClause<SlgContext, Output = R>,
    ) -> R {
        let (infer, _subst, ex_cluse) =
            InferenceTable::from_canonical(num_universes, canonical_ex_clause);
        let dyn_infer = &mut TruncatingInferenceTable::new(self.program, self.max_size, infer);
        op.with(dyn_infer, ex_cluse)
    }
}

impl<'me> TruncatingInferenceTable<'me> {
    fn new(program: &'me dyn ProgramClauseSet, max_size: usize, infer: InferenceTable) -> Self {
        Self {
            program,
            max_size,
            infer,
        }
    }
}

impl<'me> context::TruncateOps<SlgContext, SlgContext> for TruncatingInferenceTable<'me> {
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

impl<'me> context::InferenceTable<SlgContext, SlgContext> for TruncatingInferenceTable<'me> {
    fn into_hh_goal(&mut self, goal: Goal) -> HhGoal<SlgContext> {
        match goal {
            Goal::Quantified(QuantifierKind::ForAll, binders_goal) => HhGoal::ForAll(binders_goal),
            Goal::Quantified(QuantifierKind::Exists, binders_goal) => HhGoal::Exists(binders_goal),
            Goal::Implies(dg, subgoal) => HhGoal::Implies(dg, *subgoal),
            Goal::And(g1, g2) => HhGoal::And(*g1, *g2),
            Goal::Not(g1) => HhGoal::Not(*g1),
            Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })) => HhGoal::Unify((), a, b),
            Goal::Leaf(LeafGoal::DomainGoal(domain_goal)) => HhGoal::DomainGoal(domain_goal),
            Goal::CannotProve(()) => HhGoal::CannotProve,
        }
    }

    // Used by: simplify
    fn add_clauses(
        &mut self,
        env: &Arc<Environment>,
        clauses: Vec<ProgramClause>,
    ) -> Arc<Environment> {
        Environment::add_clauses(env, clauses)
    }

    fn into_goal(&self, domain_goal: DomainGoal) -> Goal {
        domain_goal.cast()
    }

    fn cannot_prove(&self) -> Goal {
        Goal::CannotProve(())
    }
}

impl<'me> context::UnificationOps<SlgContext, SlgContext> for TruncatingInferenceTable<'me> {
    fn program_clauses(
        &self,
        environment: &Arc<Environment>,
        goal: &DomainGoal,
    ) -> Vec<ProgramClause> {
        let mut clauses: Vec<_> = environment
            .clauses
            .iter()
            .filter(|&env_clause| env_clause.could_match(goal))
            .cloned()
            .collect();

        self.program
            .program_clauses_that_could_match(goal, &mut clauses);

        clauses
    }

    fn instantiate_binders_universally(&mut self, arg: &Binders<Box<Goal>>) -> Goal {
        *self.infer.instantiate_binders_universally(arg)
    }

    fn instantiate_binders_existentially(&mut self, arg: &Binders<Box<Goal>>) -> Goal {
        *self.infer.instantiate_binders_existentially(arg)
    }

    fn debug_ex_clause<'v>(&mut self, value: &'v ExClause<SlgContext>) -> Box<dyn Debug + 'v> {
        Box::new(self.infer.normalize_deep(value))
    }

    fn canonicalize_goal(&mut self, value: &InEnvironment<Goal>) -> Canonical<InEnvironment<Goal>> {
        self.infer.canonicalize(value).quantified
    }

    fn canonicalize_ex_clause(
        &mut self,
        value: &ExClause<SlgContext>,
    ) -> Canonical<ExClause<SlgContext>> {
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
        crate::infer::ucanonicalize::UniverseMap,
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
        _: (),
        a: &Parameter,
        b: &Parameter,
    ) -> Fallible<UnificationResult> {
        self.infer.unify(environment, a, b)
    }

    /// Since we do not have distinct types for the inference context and the slg-context,
    /// these conversion operations are just no-ops.q
    fn sink_answer_subset(&self, c: &Canonical<ConstrainedSubst>) -> Canonical<ConstrainedSubst> {
        c.clone()
    }

    /// Since we do not have distinct types for the inference context and the slg-context,
    /// these conversion operations are just no-ops.q
    fn lift_delayed_literal(&self, c: DelayedLiteral<SlgContext>) -> DelayedLiteral<SlgContext> {
        c
    }

    fn into_ex_clause(&mut self, result: UnificationResult, ex_clause: &mut ExClause<SlgContext>) {
        into_ex_clause(result, ex_clause)
    }
}

/// Helper function
fn into_ex_clause(result: UnificationResult, ex_clause: &mut ExClause<SlgContext>) {
    ex_clause
        .subgoals
        .extend(result.goals.into_iter().casted().map(Literal::Positive));
    ex_clause.constraints.extend(result.constraints);
}

trait SubstitutionExt {
    fn may_invalidate(&self, subst: &Canonical<Substitution>) -> bool;
}

impl SubstitutionExt for Substitution {
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
        match (&new.0, &current.0) {
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
            (_, Ty::BoundVar(_)) => {
                // If the aggregate solution already has an inference
                // variable here, then no matter what type we produce,
                // the aggregate cannot get 'more generalized' than it
                // already is. So return false, we cannot invalidate.
                //
                // (Note that "inference variables" show up as *bound
                // variables* here, because we are looking at the
                // canonical form.)
                false
            }

            (Ty::BoundVar(_), _) => {
                // If we see a type variable in the potential future
                // solution, we have to be conservative. We don't know
                // what type variable will wind up being! Remember
                // that the future solution could be any instantiation
                // of `ty0` -- or it could leave this variable
                // unbound, if the result is true for all types.
                //
                // (Note that "inference variables" show up as *bound
                // variables* here, because we are looking at the
                // canonical form.)
                true
            }

            (Ty::InferenceVar(_), _) | (_, Ty::InferenceVar(_)) => {
                panic!(
                    "unexpected free inference variable in may-invalidate: {:?} vs {:?}",
                    new, current,
                );
            }

            (Ty::Apply(apply1), Ty::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (Ty::Projection(apply1), Ty::Projection(apply2)) => {
                self.aggregate_projection_tys(apply1, apply2)
            }

            (Ty::UnselectedProjection(apply1), Ty::UnselectedProjection(apply2)) => {
                self.aggregate_unselected_projection_tys(apply1, apply2)
            }

            // For everything else, be conservative here and just say we may invalidate.
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
