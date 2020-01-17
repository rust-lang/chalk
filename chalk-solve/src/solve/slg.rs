use crate::clauses::program_clauses_for_goal;
use crate::coinductive_goal::IsCoinductive;
use crate::infer::ucanonicalize::{UCanonicalized, UniverseMap};
use crate::infer::unify::UnificationResult;
use crate::infer::InferenceTable;
use crate::solve::truncate::{self, Truncated};
use crate::solve::Solution;
use crate::RustIrDatabase;
use chalk_derive::HasTypeFamily;
use chalk_engine::context;
use chalk_engine::context::Floundered;
use chalk_engine::fallible::Fallible;
use chalk_engine::hh::HhGoal;
use chalk_engine::{CompleteAnswer, ExClause, Literal};
use chalk_ir::cast::Cast;
use chalk_ir::cast::Caster;
use chalk_ir::could_match::CouldMatch;
use chalk_ir::family::HasTypeFamily;
use chalk_ir::family::TypeFamily;
use chalk_ir::*;

use std::fmt::Debug;
use std::marker::PhantomData;

mod aggregate;
mod resolvent;

#[derive(Clone, Debug, HasTypeFamily)]
pub(crate) struct SlgContext<TF: TypeFamily> {
    max_size: usize,
    /// The expected number of answers for a solution.
    /// Only really sseful for tests, since `make_solution`
    /// will panic if the number of cached answers does not
    /// equal this when a solution is made.
    expected_answers: Option<usize>,
    phantom: PhantomData<TF>,
}

impl<TF: TypeFamily> SlgContext<TF> {
    pub(crate) fn new(max_size: usize, expected_answers: Option<usize>) -> SlgContext<TF> {
        SlgContext {
            max_size,
            expected_answers,
            phantom: PhantomData,
        }
    }

    pub(crate) fn ops<'p>(&self, program: &'p dyn RustIrDatabase<TF>) -> SlgContextOps<'p, TF> {
        SlgContextOps {
            program,
            max_size: self.max_size,
            expected_answers: self.expected_answers,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SlgContextOps<'me, TF: TypeFamily> {
    program: &'me dyn RustIrDatabase<TF>,
    max_size: usize,
    expected_answers: Option<usize>,
}

#[derive(Clone)]
pub struct TruncatingInferenceTable<TF: TypeFamily> {
    max_size: usize,
    infer: InferenceTable<TF>,
}

impl<TF: TypeFamily> context::Context for SlgContext<TF> {
    type CanonicalGoalInEnvironment = Canonical<InEnvironment<Goal<TF>>>;
    type CanonicalExClause = Canonical<ExClause<Self>>;
    type UCanonicalGoalInEnvironment = UCanonical<InEnvironment<Goal<TF>>>;
    type UniverseMap = UniverseMap;
    type InferenceNormalizedSubst = Substitution<TF>;
    type Solution = Solution<TF>;
    type InferenceTable = TruncatingInferenceTable<TF>;
    type Environment = Environment<TF>;
    type DomainGoal = DomainGoal<TF>;
    type Goal = Goal<TF>;
    type BindersGoal = Binders<Goal<TF>>;
    type Parameter = Parameter<TF>;
    type ProgramClause = ProgramClause<TF>;
    type ProgramClauses = Vec<ProgramClause<TF>>;
    type CanonicalConstrainedSubst = Canonical<ConstrainedSubst<TF>>;
    type CanonicalAnswerSubst = Canonical<AnswerSubst<TF>>;
    type GoalInEnvironment = InEnvironment<Goal<TF>>;
    type Substitution = Substitution<TF>;
    type RegionConstraint = InEnvironment<Constraint<TF>>;
    type Variance = ();

    fn goal_in_environment(
        environment: &Environment<TF>,
        goal: Goal<TF>,
    ) -> InEnvironment<Goal<TF>> {
        InEnvironment::new(environment, goal)
    }

    fn inference_normalized_subst_from_ex_clause(
        canon_ex_clause: &Canonical<ExClause<SlgContext<TF>>>,
    ) -> &Substitution<TF> {
        &canon_ex_clause.value.subst
    }

    fn empty_constraints(ccs: &Canonical<AnswerSubst<TF>>) -> bool {
        ccs.value.constraints.is_empty()
    }

    fn inference_normalized_subst_from_subst(
        ccs: &Canonical<AnswerSubst<TF>>,
    ) -> &Substitution<TF> {
        &ccs.value.subst
    }

    fn canonical(
        u_canon: &UCanonical<InEnvironment<Goal<TF>>>,
    ) -> &Canonical<InEnvironment<Goal<TF>>> {
        &u_canon.canonical
    }

    fn is_trivial_substitution(
        u_canon: &UCanonical<InEnvironment<Goal<TF>>>,
        canonical_subst: &Canonical<AnswerSubst<TF>>,
    ) -> bool {
        u_canon.is_trivial_substitution(canonical_subst)
    }

    fn has_delayed_subgoals(canonical_subst: &Canonical<AnswerSubst<TF>>) -> bool {
        !canonical_subst.value.delayed_subgoals.is_empty()
    }

    fn num_universes(u_canon: &UCanonical<InEnvironment<Goal<TF>>>) -> usize {
        u_canon.universes
    }

    fn canonical_constrained_subst_from_canonical_constrained_answer(
        canonical_subst: &Canonical<AnswerSubst<TF>>,
    ) -> Canonical<ConstrainedSubst<TF>> {
        Canonical {
            binders: canonical_subst.binders.clone(),
            value: ConstrainedSubst {
                subst: canonical_subst.value.subst.clone(),
                constraints: canonical_subst.value.constraints.clone(),
            },
        }
    }

    fn map_goal_from_canonical(
        map: &UniverseMap,
        value: &Canonical<InEnvironment<Goal<TF>>>,
    ) -> Canonical<InEnvironment<Goal<TF>>> {
        map.map_from_canonical(value)
    }

    fn map_subst_from_canonical(
        map: &UniverseMap,
        value: &Canonical<AnswerSubst<TF>>,
    ) -> Canonical<AnswerSubst<TF>> {
        map.map_from_canonical(value)
    }

    fn goal_from_goal_in_environment(goal: &InEnvironment<Goal<TF>>) -> &Goal<TF> {
        &goal.goal
    }

    fn identity_constrained_subst(
        goal: &UCanonical<InEnvironment<Goal<TF>>>,
    ) -> Canonical<ConstrainedSubst<TF>> {
        let (mut infer, subst, _) = InferenceTable::from_canonical(goal.universes, &goal.canonical);
        infer
            .canonicalize(&ConstrainedSubst {
                subst,
                constraints: vec![],
            })
            .quantified
    }

    fn into_hh_goal(goal: Goal<TF>) -> HhGoal<SlgContext<TF>> {
        match goal.data().clone() {
            GoalData::Quantified(QuantifierKind::ForAll, binders_goal) => {
                HhGoal::ForAll(binders_goal)
            }
            GoalData::Quantified(QuantifierKind::Exists, binders_goal) => {
                HhGoal::Exists(binders_goal)
            }
            GoalData::Implies(dg, subgoal) => HhGoal::Implies(dg, subgoal),
            GoalData::All(goals) => HhGoal::All(goals),
            GoalData::Not(g1) => HhGoal::Not(g1),
            GoalData::EqGoal(EqGoal { a, b }) => HhGoal::Unify((), a, b),
            GoalData::DomainGoal(domain_goal) => HhGoal::DomainGoal(domain_goal),
            GoalData::CannotProve(()) => HhGoal::CannotProve,
        }
    }

    // Used by: simplify
    fn add_clauses(env: &Environment<TF>, clauses: Vec<ProgramClause<TF>>) -> Environment<TF> {
        Environment::add_clauses(env, clauses)
    }

    fn into_goal(domain_goal: DomainGoal<TF>) -> Goal<TF> {
        domain_goal.cast()
    }

    // Used by: logic
    fn next_subgoal_index(ex_clause: &ExClause<SlgContext<TF>>) -> usize {
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

impl<'me, TF: TypeFamily> context::ContextOps<SlgContext<TF>> for SlgContextOps<'me, TF> {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal<TF>>>) -> bool {
        goal.is_coinductive(self.program)
    }

    fn program_clauses(
        &self,
        environment: &Environment<TF>,
        goal: &DomainGoal<TF>,
        infer: &mut TruncatingInferenceTable<TF>,
    ) -> Result<Vec<ProgramClause<TF>>, Floundered> {
        // Look for floundering goals:
        match goal {
            // Check for a goal like `?T: Foo` where `Foo` is not enumerable.
            DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
                let trait_datum = self.program.trait_datum(trait_ref.trait_id);
                if trait_datum.is_non_enumerable_trait() || trait_datum.is_auto_trait() {
                    let self_ty = trait_ref.self_type_parameter();
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
            | DomainGoal::IsLocal(ty) => match ty.data() {
                TyData::InferenceVar(_) => return Err(Floundered),
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

    fn instantiate_ucanonical_goal(
        &self,
        arg: &UCanonical<InEnvironment<Goal<TF>>>,
    ) -> (
        TruncatingInferenceTable<TF>,
        Substitution<TF>,
        Environment<TF>,
        Goal<TF>,
    ) {
        let (infer, subst, InEnvironment { environment, goal }) =
            InferenceTable::from_canonical(arg.universes, &arg.canonical);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        (infer_table, subst, environment, goal)
    }

    fn instantiate_ex_clause(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<SlgContext<TF>>>,
    ) -> (TruncatingInferenceTable<TF>, ExClause<SlgContext<TF>>) {
        let (infer, _subst, ex_cluse) =
            InferenceTable::from_canonical(num_universes, canonical_ex_clause);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        (infer_table, ex_cluse)
    }

    fn instantiate_answer_subst(
        &self,
        num_universes: usize,
        answer: &Canonical<AnswerSubst<TF>>,
    ) -> (
        TruncatingInferenceTable<TF>,
        Substitution<TF>,
        Vec<InEnvironment<Constraint<TF>>>,
        Vec<InEnvironment<Goal<TF>>>,
    ) {
        let (
            infer,
            _subst,
            AnswerSubst {
                subst,
                constraints,
                delayed_subgoals,
            },
        ) = InferenceTable::from_canonical(num_universes, answer);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        (infer_table, subst, constraints, delayed_subgoals)
    }

    fn constrained_subst_from_answer(
        &self,
        answer: CompleteAnswer<SlgContext<TF>>,
    ) -> Canonical<ConstrainedSubst<TF>> {
        let CompleteAnswer { subst, .. } = answer;
        subst
    }
}

impl<TF: TypeFamily> TruncatingInferenceTable<TF> {
    fn new(max_size: usize, infer: InferenceTable<TF>) -> Self {
        Self { max_size, infer }
    }
}

impl<TF: TypeFamily> context::TruncateOps<SlgContext<TF>> for TruncatingInferenceTable<TF> {
    fn truncate_goal(
        &mut self,
        subgoal: &InEnvironment<Goal<TF>>,
    ) -> Option<InEnvironment<Goal<TF>>> {
        // We only want to truncate the goal itself. We keep the environment intact.
        // See rust-lang/chalk#280
        let InEnvironment { environment, goal } = subgoal;
        let Truncated { overflow, value } =
            truncate::truncate(&mut self.infer, self.max_size, goal);
        if overflow {
            Some(InEnvironment {
                environment: environment.clone(),
                goal: value,
            })
        } else {
            None
        }
    }

    fn truncate_answer(&mut self, subst: &Substitution<TF>) -> Option<Substitution<TF>> {
        let Truncated { overflow, value } =
            truncate::truncate(&mut self.infer, self.max_size, subst);
        if overflow {
            Some(value)
        } else {
            None
        }
    }
}

impl<TF: TypeFamily> context::InferenceTable<SlgContext<TF>> for TruncatingInferenceTable<TF> {}

impl<TF: TypeFamily> context::UnificationOps<SlgContext<TF>> for TruncatingInferenceTable<TF> {
    fn instantiate_binders_universally(&mut self, arg: &Binders<Goal<TF>>) -> Goal<TF> {
        self.infer.instantiate_binders_universally(arg)
    }

    fn instantiate_binders_existentially(&mut self, arg: &Binders<Goal<TF>>) -> Goal<TF> {
        self.infer.instantiate_binders_existentially(arg)
    }

    fn debug_ex_clause<'v>(&mut self, value: &'v ExClause<SlgContext<TF>>) -> Box<dyn Debug + 'v> {
        Box::new(self.infer.normalize_deep(value))
    }

    fn fully_canonicalize_goal(
        &mut self,
        value: &InEnvironment<Goal<TF>>,
    ) -> (UCanonical<InEnvironment<Goal<TF>>>, UniverseMap) {
        let canonicalized_goal = self.infer.canonicalize(value).quantified;
        let UCanonicalized {
            quantified,
            universes,
        } = self.infer.u_canonicalize(&canonicalized_goal);
        (quantified, universes)
    }

    fn canonicalize_ex_clause(
        &mut self,
        value: &ExClause<SlgContext<TF>>,
    ) -> Canonical<ExClause<SlgContext<TF>>> {
        self.infer.canonicalize(value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        subst: Substitution<TF>,
        constraints: Vec<InEnvironment<Constraint<TF>>>,
    ) -> Canonical<ConstrainedSubst<TF>> {
        self.infer
            .canonicalize(&ConstrainedSubst { subst, constraints })
            .quantified
    }

    fn canonicalize_answer_subst(
        &mut self,
        subst: Substitution<TF>,
        constraints: Vec<InEnvironment<Constraint<TF>>>,
        delayed_subgoals: Vec<InEnvironment<Goal<TF>>>,
    ) -> Canonical<AnswerSubst<TF>> {
        self.infer
            .canonicalize(&AnswerSubst {
                subst,
                constraints,
                delayed_subgoals,
            })
            .quantified
    }

    fn invert_goal(&mut self, value: &InEnvironment<Goal<TF>>) -> Option<InEnvironment<Goal<TF>>> {
        self.infer.invert(value)
    }

    fn unify_parameters_into_ex_clause(
        &mut self,
        environment: &Environment<TF>,
        _: (),
        a: &Parameter<TF>,
        b: &Parameter<TF>,
        ex_clause: &mut ExClause<SlgContext<TF>>,
    ) -> Fallible<()> {
        let result = self.infer.unify(environment, a, b)?;
        Ok(into_ex_clause(result, ex_clause))
    }
}

/// Helper function
fn into_ex_clause<TF: TypeFamily>(
    result: UnificationResult<TF>,
    ex_clause: &mut ExClause<SlgContext<TF>>,
) {
    ex_clause
        .subgoals
        .extend(result.goals.into_iter().casted().map(Literal::Positive));
    ex_clause.constraints.extend(result.constraints);
}

trait SubstitutionExt<TF: TypeFamily> {
    fn may_invalidate(&self, subst: &Canonical<Substitution<TF>>) -> bool;
}

impl<TF: TypeFamily> SubstitutionExt<TF> for Substitution<TF> {
    fn may_invalidate(&self, subst: &Canonical<Substitution<TF>>) -> bool {
        self.iter()
            .zip(subst.value.iter())
            .any(|(new, current)| MayInvalidate.aggregate_parameters(new, current))
    }
}

// This is a struct in case we need to add state at any point like in AntiUnifier
struct MayInvalidate;

impl MayInvalidate {
    fn aggregate_parameters<TF: TypeFamily>(
        &mut self,
        new: &Parameter<TF>,
        current: &Parameter<TF>,
    ) -> bool {
        match (new.data(), current.data()) {
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
    fn aggregate_tys<TF: TypeFamily>(&mut self, new: &Ty<TF>, current: &Ty<TF>) -> bool {
        match (new.data(), current.data()) {
            (_, TyData::BoundVar(_)) => {
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

            (TyData::BoundVar(_), _) => {
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

            (TyData::InferenceVar(_), _) | (_, TyData::InferenceVar(_)) => {
                panic!(
                    "unexpected free inference variable in may-invalidate: {:?} vs {:?}",
                    new, current,
                );
            }

            (TyData::Apply(apply1), TyData::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (TyData::Placeholder(p1), TyData::Placeholder(p2)) => {
                self.aggregate_placeholder_tys(p1, p2)
            }

            (TyData::Projection(apply1), TyData::Projection(apply2)) => {
                self.aggregate_projection_tys(apply1, apply2)
            }

            // For everything else, be conservative here and just say we may invalidate.
            (TyData::Function(_), _)
            | (TyData::Dyn(_), _)
            | (TyData::Apply(_), _)
            | (TyData::Placeholder(_), _)
            | (TyData::Projection(_), _) => true,
        }
    }

    fn aggregate_lifetimes<TF: TypeFamily>(&mut self, _: &Lifetime<TF>, _: &Lifetime<TF>) -> bool {
        true
    }

    fn aggregate_application_tys<TF: TypeFamily>(
        &mut self,
        new: &ApplicationTy<TF>,
        current: &ApplicationTy<TF>,
    ) -> bool {
        let ApplicationTy {
            name: new_name,
            substitution: new_substitution,
        } = new;
        let ApplicationTy {
            name: current_name,
            substitution: current_substitution,
        } = current;

        self.aggregate_name_and_substs(
            new_name,
            new_substitution,
            current_name,
            current_substitution,
        )
    }

    fn aggregate_placeholder_tys(
        &mut self,
        new: &PlaceholderIndex,
        current: &PlaceholderIndex,
    ) -> bool {
        new != current
    }

    fn aggregate_projection_tys<TF: TypeFamily>(
        &mut self,
        new: &ProjectionTy<TF>,
        current: &ProjectionTy<TF>,
    ) -> bool {
        let ProjectionTy {
            associated_ty_id: new_name,
            substitution: new_substitution,
        } = new;
        let ProjectionTy {
            associated_ty_id: current_name,
            substitution: current_substitution,
        } = current;

        self.aggregate_name_and_substs(
            new_name,
            new_substitution,
            current_name,
            current_substitution,
        )
    }

    fn aggregate_name_and_substs<N, TF>(
        &mut self,
        new_name: N,
        new_substitution: &Substitution<TF>,
        current_name: N,
        current_substitution: &Substitution<TF>,
    ) -> bool
    where
        N: Copy + Eq + Debug,
        TF: TypeFamily,
    {
        if new_name != current_name {
            return true;
        }

        let name = new_name;

        assert_eq!(
            new_substitution.len(),
            current_substitution.len(),
            "does {:?} take {} substitution or {}? can't both be right",
            name,
            new_substitution.len(),
            current_substitution.len()
        );

        new_substitution
            .iter()
            .zip(current_substitution)
            .any(|(new, current)| self.aggregate_parameters(new, current))
    }
}
