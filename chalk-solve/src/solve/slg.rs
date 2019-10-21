use crate::clauses::program_clauses_for_goal;
use crate::coinductive_goal::IsCoinductive;
use crate::infer::ucanonicalize::{UCanonicalized, UniverseMap};
use crate::infer::unify::UnificationResult;
use crate::infer::InferenceTable;
use crate::solve::truncate::{self, Truncated};
use crate::solve::Solution;
use crate::RustIrDatabase;
use chalk_engine::context::Floundered;
use chalk_engine::fallible::Fallible;
use chalk_ir::cast::Cast;
use chalk_ir::cast::Caster;
use chalk_ir::could_match::CouldMatch;
use chalk_ir::family::ChalkIr;
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

    pub(crate) fn ops<'p>(&self, program: &'p dyn RustIrDatabase) -> SlgContextOps<'p> {
        SlgContextOps {
            program,
            max_size: self.max_size,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SlgContextOps<'me> {
    program: &'me dyn RustIrDatabase,
    max_size: usize,
}

pub struct TruncatingInferenceTable {
    max_size: usize,
    infer: InferenceTable,
}

impl context::Context for SlgContext {
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal<ChalkIr>>>;
    type CanonicalExClause = Canonical<ExClause<Self>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal<ChalkIr>>>;
    type UniverseMap = UniverseMap;
    type InferenceNormalizedSubst = Substitution<ChalkIr>;
    type Solution = Solution;
    type InferenceTable = TruncatingInferenceTable;
    type Environment = Arc<Environment<ChalkIr>>;
    type DomainGoal = DomainGoal<ChalkIr>;
    type Goal = Goal<ChalkIr>;
    type BindersGoal = Binders<Box<Goal<ChalkIr>>>;
    type Parameter = Parameter<ChalkIr>;
    type ProgramClause = ProgramClause<ChalkIr>;
    type ProgramClauses = Vec<ProgramClause<ChalkIr>>;
    type UnificationResult = UnificationResult;
    type CanonicalConstrainedSubst = Canonical<ConstrainedSubst<ChalkIr>>;
    type GoalInEnvironment = InEnvironment<Goal<ChalkIr>>;
    type Substitution = Substitution<ChalkIr>;
    type RegionConstraint = InEnvironment<Constraint<ChalkIr>>;
    type Variance = ();

    fn goal_in_environment(
        environment: &Arc<Environment<ChalkIr>>,
        goal: Goal<ChalkIr>,
    ) -> InEnvironment<Goal<ChalkIr>> {
        InEnvironment::new(environment, goal)
    }

    fn inference_normalized_subst_from_ex_clause(
        canon_ex_clause: &Canonical<ExClause<SlgContext>>,
    ) -> &Substitution<ChalkIr> {
        &canon_ex_clause.value.subst
    }

    fn empty_constraints(ccs: &Canonical<ConstrainedSubst<ChalkIr>>) -> bool {
        ccs.value.constraints.is_empty()
    }

    fn inference_normalized_subst_from_subst(
        ccs: &Canonical<ConstrainedSubst<ChalkIr>>,
    ) -> &Substitution<ChalkIr> {
        &ccs.value.subst
    }

    fn canonical(
        u_canon: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> &Canonical<InEnvironment<Goal<ChalkIr>>> {
        &u_canon.canonical
    }

    fn is_trivial_substitution(
        u_canon: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
        canonical_subst: &Canonical<ConstrainedSubst<ChalkIr>>,
    ) -> bool {
        u_canon.is_trivial_substitution(canonical_subst)
    }

    fn num_universes(u_canon: &UCanonical<InEnvironment<Goal<ChalkIr>>>) -> usize {
        u_canon.universes
    }

    fn map_goal_from_canonical(
        map: &UniverseMap,
        value: &Canonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> Canonical<InEnvironment<Goal<ChalkIr>>> {
        map.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        map: &UniverseMap,
        value: &Canonical<ConstrainedSubst<ChalkIr>>,
    ) -> Canonical<ConstrainedSubst<ChalkIr>> {
        map.map_from_canonical(value)
    }
}

impl<'me> context::ContextOps<SlgContext> for SlgContextOps<'me> {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>) -> bool {
        goal.is_coinductive(self.program)
    }

    fn identity_constrained_subst(
        &self,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> Canonical<ConstrainedSubst<ChalkIr>> {
        let (mut infer, subst, _) = InferenceTable::from_canonical(goal.universes, &goal.canonical);
        infer
            .canonicalize(&ConstrainedSubst {
                subst,
                constraints: vec![],
            })
            .quantified
    }

    fn program_clauses(
        &self,
        environment: &Arc<Environment<ChalkIr>>,
        goal: &DomainGoal<ChalkIr>,
        infer: &mut TruncatingInferenceTable,
    ) -> Result<Vec<ProgramClause<ChalkIr>>, Floundered> {
        // Look for floundering goals:
        match goal {
            // Check for a goal like `?T: Foo` where `Foo` is not enumerable.
            DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
                let trait_datum = self.program.trait_datum(trait_ref.trait_id);
                if trait_datum.is_non_enumerable_trait() || trait_datum.is_auto_trait() {
                    let self_ty = trait_ref.self_type_parameter().unwrap();
                    if let Some(v) = self_ty.inference_var() {
                        if !infer.infer.var_is_bound(v) {
                            return Err(Floundered);
                        }
                    }
                }
            }

            DomainGoal::WellFormed(WellFormed::Ty(ty))
            | DomainGoal::IsUpstream(ty)
            | DomainGoal::DownstreamType(ty)
            | DomainGoal::IsFullyVisible(ty)
            | DomainGoal::IsLocal(ty) => match ty {
                Ty::InferenceVar(_) => return Err(Floundered),
                _ => {}
            },

            _ => {}
        }

        let mut clauses: Vec<_> = program_clauses_for_goal(self.program, environment, goal);

        clauses.extend(
            environment
                .clauses
                .iter()
                .filter(|&env_clause| env_clause.could_match(goal))
                .cloned(),
        );

        Ok(clauses)
    }

    fn instantiate_ucanonical_goal<R>(
        &self,
        arg: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
        op: impl FnOnce(
            TruncatingInferenceTable,
            Substitution<ChalkIr>,
            Arc<Environment<ChalkIr>>,
            Goal<ChalkIr>,
        ) -> R,
    ) -> R {
        let (infer, subst, InEnvironment { environment, goal }) =
            InferenceTable::from_canonical(arg.universes, &arg.canonical);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        op(infer_table, subst, environment, goal)
    }

    fn instantiate_ex_clause<R>(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<SlgContext>>,
        op: impl FnOnce(TruncatingInferenceTable, ExClause<SlgContext>) -> R,
    ) -> R {
        let (infer, _subst, ex_cluse) =
            InferenceTable::from_canonical(num_universes, canonical_ex_clause);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        op(infer_table, ex_cluse)
    }
}

impl TruncatingInferenceTable {
    fn new(max_size: usize, infer: InferenceTable) -> Self {
        Self {
            max_size,
            infer,
        }
    }
}

impl context::TruncateOps<SlgContext> for TruncatingInferenceTable {
    fn truncate_goal(
        &mut self,
        subgoal: &InEnvironment<Goal<ChalkIr>>
    ) -> Option<InEnvironment<Goal<ChalkIr>>> {
        let Truncated { overflow, value } =
            truncate::truncate(&mut self.infer, self.max_size, subgoal);
        if overflow {
            Some(value)
        } else {
            None
        }
    }

    fn truncate_answer(&mut self, subst: &Substitution<ChalkIr>) -> Option<Substitution<ChalkIr>> {
        let Truncated { overflow, value } =
            truncate::truncate(&mut self.infer, self.max_size, subst);
        if overflow {
            Some(value)
        } else {
            None
        }
    }
}

impl context::InferenceTable<SlgContext> for TruncatingInferenceTable {
    fn into_hh_goal(&mut self, goal: Goal<ChalkIr>) -> HhGoal<SlgContext> {
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
        env: &Arc<Environment<ChalkIr>>,
        clauses: Vec<ProgramClause<ChalkIr>>,
    ) -> Arc<Environment<ChalkIr>> {
        Environment::add_clauses(env, clauses)
    }

    fn into_goal(&self, domain_goal: DomainGoal<ChalkIr>) -> Goal<ChalkIr> {
        domain_goal.cast()
    }

    fn cannot_prove(&self) -> Goal<ChalkIr> {
        Goal::CannotProve(())
    }

    // Used by: logic
    fn next_subgoal_index(&mut self, ex_clause: &ExClause<SlgContext>) -> usize {
        // For now, we always pick the last subgoal in the
        // list.
        //
        // FIXME(rust-lang-nursery/chalk#80) -- we should be more
        // selective. For example, we don't want to pick a
        // negative literal that will flounder, and we don't want
        // to pick things like `?T: Sized` if we can help it.
        ex_clause.subgoals.len() - 1
    }
}

impl context::UnificationOps<SlgContext> for TruncatingInferenceTable {
    fn instantiate_binders_universally(
        &mut self,
        arg: &Binders<Box<Goal<ChalkIr>>>,
    ) -> Goal<ChalkIr> {
        *self.infer.instantiate_binders_universally(arg)
    }

    fn instantiate_binders_existentially(
        &mut self,
        arg: &Binders<Box<Goal<ChalkIr>>>,
    ) -> Goal<ChalkIr> {
        *self.infer.instantiate_binders_existentially(arg)
    }

    fn debug_ex_clause<'v>(&mut self, value: &'v ExClause<SlgContext>) -> Box<dyn Debug + 'v> {
        Box::new(self.infer.normalize_deep(value))
    }

    fn canonicalize_goal(
        &mut self,
        value: &InEnvironment<Goal<ChalkIr>>,
    ) -> Canonical<InEnvironment<Goal<ChalkIr>>> {
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
        subst: Substitution<ChalkIr>,
        constraints: Vec<InEnvironment<Constraint<ChalkIr>>>,
    ) -> Canonical<ConstrainedSubst<ChalkIr>> {
        self.infer
            .canonicalize(&ConstrainedSubst { subst, constraints })
            .quantified
    }

    fn u_canonicalize_goal(
        &mut self,
        value: &Canonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> (
        UCanonical<InEnvironment<Goal<ChalkIr>>>,
        crate::infer::ucanonicalize::UniverseMap,
    ) {
        let UCanonicalized {
            quantified,
            universes,
        } = self.infer.u_canonicalize(value);
        (quantified, universes)
    }

    fn invert_goal(
        &mut self,
        value: &InEnvironment<Goal<ChalkIr>>,
    ) -> Option<InEnvironment<Goal<ChalkIr>>> {
        self.infer.invert(value)
    }

    fn unify_parameters(
        &mut self,
        environment: &Arc<Environment<ChalkIr>>,
        _: (),
        a: &Parameter<ChalkIr>,
        b: &Parameter<ChalkIr>,
    ) -> Fallible<UnificationResult> {
        self.infer.unify(environment, a, b)
    }

    /// Since we do not have distinct types for the inference context and the slg-context,
    /// these conversion operations are just no-ops.q
    fn sink_answer_subset(
        &self,
        c: &Canonical<ConstrainedSubst<ChalkIr>>,
    ) -> Canonical<ConstrainedSubst<ChalkIr>> {
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
    fn may_invalidate(&self, subst: &Canonical<Substitution<ChalkIr>>) -> bool;
}

impl SubstitutionExt for Substitution<ChalkIr> {
    fn may_invalidate(&self, subst: &Canonical<Substitution<ChalkIr>>) -> bool {
        self.parameters
            .iter()
            .zip(&subst.value.parameters)
            .any(|(new, current)| MayInvalidate.aggregate_parameters(new, current))
    }
}

// This is a struct in case we need to add state at any point like in AntiUnifier
struct MayInvalidate;

impl MayInvalidate {
    fn aggregate_parameters(
        &mut self,
        new: &Parameter<ChalkIr>,
        current: &Parameter<ChalkIr>,
    ) -> bool {
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
    fn aggregate_tys(&mut self, new: &Ty<ChalkIr>, current: &Ty<ChalkIr>) -> bool {
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

            // For everything else, be conservative here and just say we may invalidate.
            (Ty::ForAll(_), _)
            | (Ty::Dyn(_), _)
            | (Ty::Opaque(_), _)
            | (Ty::Apply(_), _)
            | (Ty::Projection(_), _) => true,
        }
    }

    fn aggregate_lifetimes(&mut self, _: &Lifetime<ChalkIr>, _: &Lifetime<ChalkIr>) -> bool {
        true
    }

    fn aggregate_application_tys(
        &mut self,
        new: &ApplicationTy<ChalkIr>,
        current: &ApplicationTy<ChalkIr>,
    ) -> bool {
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

    fn aggregate_projection_tys(
        &mut self,
        new: &ProjectionTy<ChalkIr>,
        current: &ProjectionTy<ChalkIr>,
    ) -> bool {
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

    fn aggregate_name_and_substs<N>(
        &mut self,
        new_name: N,
        new_parameters: &[Parameter<ChalkIr>],
        current_name: N,
        current_parameters: &[Parameter<ChalkIr>],
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
