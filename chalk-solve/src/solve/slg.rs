use crate::clauses::program_clauses_for_goal;
use crate::coinductive_goal::IsCoinductive;
use crate::infer::ucanonicalize::UCanonicalized;
use crate::infer::unify::UnificationResult;
use crate::infer::InferenceTable;
use crate::solve::truncate;
use crate::RustIrDatabase;
use chalk_derive::HasInterner;
use chalk_engine::context;
use chalk_engine::{ExClause, Literal};
use chalk_ir::cast::Cast;
use chalk_ir::cast::Caster;
use chalk_ir::interner::Interner;
use chalk_ir::*;

use std::fmt::{Debug, Display};
use std::marker::PhantomData;

pub(crate) mod aggregate;
mod resolvent;

#[derive(Debug)]
pub enum SubstitutionResult<S> {
    Definite(S),
    Ambiguous(S),
    Floundered,
}

impl<S> SubstitutionResult<S> {
    pub fn as_ref(&self) -> SubstitutionResult<&S> {
        match self {
            SubstitutionResult::Definite(subst) => SubstitutionResult::Definite(subst),
            SubstitutionResult::Ambiguous(subst) => SubstitutionResult::Ambiguous(subst),
            SubstitutionResult::Floundered => SubstitutionResult::Floundered,
        }
    }
    pub fn map<U, F: FnOnce(S) -> U>(self, f: F) -> SubstitutionResult<U> {
        match self {
            SubstitutionResult::Definite(subst) => SubstitutionResult::Definite(f(subst)),
            SubstitutionResult::Ambiguous(subst) => SubstitutionResult::Ambiguous(f(subst)),
            SubstitutionResult::Floundered => SubstitutionResult::Floundered,
        }
    }
}

impl<S: Display> Display for SubstitutionResult<S> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstitutionResult::Definite(subst) => write!(fmt, "{}", subst),
            SubstitutionResult::Ambiguous(subst) => write!(fmt, "Ambiguous({})", subst),
            SubstitutionResult::Floundered => write!(fmt, "Floundered"),
        }
    }
}

#[derive(Clone, Debug, HasInterner)]
pub(crate) struct SlgContext<I: Interner> {
    phantom: PhantomData<I>,
}

#[derive(Clone, Debug)]
pub(crate) struct SlgContextOps<'me, I: Interner> {
    program: &'me dyn RustIrDatabase<I>,
    max_size: usize,
    expected_answers: Option<usize>,
}

impl<I: Interner> SlgContextOps<'_, I> {
    pub(crate) fn new<'p>(
        program: &'p dyn RustIrDatabase<I>,
        max_size: usize,
        expected_answers: Option<usize>,
    ) -> SlgContextOps<'p, I> {
        SlgContextOps {
            program,
            max_size,
            expected_answers,
        }
    }
}

#[derive(Clone)]
pub struct TruncatingInferenceTable<I: Interner> {
    max_size: usize,
    infer: InferenceTable<I>,
}

impl<I: Interner> context::Context<I> for SlgContext<I> {
    type InferenceTable = TruncatingInferenceTable<I>;

    // Used by: logic
    fn next_subgoal_index(ex_clause: &ExClause<I>) -> usize {
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

impl<'me, I: Interner> context::ContextOps<I, SlgContext<I>> for SlgContextOps<'me, I> {
    fn is_coinductive(&self, goal: &UCanonical<InEnvironment<Goal<I>>>) -> bool {
        goal.is_coinductive(self.program)
    }

    fn map_goal_from_canonical(
        &self,
        map: &UniverseMap,
        value: &Canonical<InEnvironment<Goal<I>>>,
    ) -> Canonical<InEnvironment<Goal<I>>> {
        use crate::infer::ucanonicalize::UniverseMapExt;
        map.map_from_canonical(self.program.interner(), value)
    }

    fn map_subst_from_canonical(
        &self,
        map: &UniverseMap,
        value: &Canonical<AnswerSubst<I>>,
    ) -> Canonical<AnswerSubst<I>> {
        use crate::infer::ucanonicalize::UniverseMapExt;
        map.map_from_canonical(self.program.interner(), value)
    }

    fn program_clauses(
        &self,
        environment: &Environment<I>,
        goal: &DomainGoal<I>,
        _infer: &mut TruncatingInferenceTable<I>,
    ) -> Result<Vec<ProgramClause<I>>, Floundered> {
        let clauses: Vec<_> = program_clauses_for_goal(self.program, environment, goal)?;

        Ok(clauses)
    }

    // Used by: simplify
    fn add_clauses(&self, env: &Environment<I>, clauses: ProgramClauses<I>) -> Environment<I> {
        let interner = self.interner();
        env.add_clauses(interner, clauses.iter(interner).cloned())
    }

    fn instantiate_ucanonical_goal(
        &self,
        arg: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> (
        TruncatingInferenceTable<I>,
        Substitution<I>,
        Environment<I>,
        Goal<I>,
    ) {
        let (infer, subst, InEnvironment { environment, goal }) =
            InferenceTable::from_canonical(self.program.interner(), arg.universes, &arg.canonical);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        (infer_table, subst, environment, goal)
    }

    fn instantiate_ex_clause(
        &self,
        num_universes: usize,
        canonical_ex_clause: &Canonical<ExClause<I>>,
    ) -> (TruncatingInferenceTable<I>, ExClause<I>) {
        let (infer, _subst, ex_cluse) = InferenceTable::from_canonical(
            self.program.interner(),
            num_universes,
            canonical_ex_clause,
        );
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        (infer_table, ex_cluse)
    }

    fn instantiate_answer_subst(
        &self,
        num_universes: usize,
        answer: &Canonical<AnswerSubst<I>>,
    ) -> (
        TruncatingInferenceTable<I>,
        Substitution<I>,
        Vec<InEnvironment<Constraint<I>>>,
        Vec<InEnvironment<Goal<I>>>,
    ) {
        let (
            infer,
            _subst,
            AnswerSubst {
                subst,
                constraints,
                delayed_subgoals,
            },
        ) = InferenceTable::from_canonical(self.program.interner(), num_universes, answer);
        let infer_table = TruncatingInferenceTable::new(self.max_size, infer);
        (infer_table, subst, constraints, delayed_subgoals)
    }

    fn identity_constrained_subst(
        &self,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Canonical<ConstrainedSubst<I>> {
        let (mut infer, subst, _) = InferenceTable::from_canonical(
            self.program.interner(),
            goal.universes,
            &goal.canonical,
        );
        infer
            .canonicalize(
                self.program.interner(),
                &ConstrainedSubst {
                    subst,
                    constraints: vec![],
                },
            )
            .quantified
    }

    fn interner(&self) -> &I {
        self.program.interner()
    }

    fn into_goal(&self, domain_goal: DomainGoal<I>) -> Goal<I> {
        domain_goal.cast(self.program.interner())
    }

    fn is_trivial_constrained_substitution(
        &self,
        constrained_subst: &Canonical<ConstrainedSubst<I>>,
    ) -> bool {
        let interner = self.interner();
        constrained_subst.value.subst.is_identity_subst(interner)
    }

    fn is_trivial_substitution(
        &self,
        u_canon: &UCanonical<InEnvironment<Goal<I>>>,
        canonical_subst: &Canonical<AnswerSubst<I>>,
    ) -> bool {
        let interner = self.interner();
        u_canon.is_trivial_substitution(interner, canonical_subst)
    }
}

impl<I: Interner> TruncatingInferenceTable<I> {
    fn new(max_size: usize, infer: InferenceTable<I>) -> Self {
        Self { max_size, infer }
    }
}

impl<I: Interner> context::TruncateOps<I, SlgContext<I>> for TruncatingInferenceTable<I> {
    fn goal_needs_truncation(&mut self, interner: &I, subgoal: &InEnvironment<Goal<I>>) -> bool {
        truncate::needs_truncation(interner, &mut self.infer, self.max_size, &subgoal)
    }

    fn answer_needs_truncation(&mut self, interner: &I, subst: &Substitution<I>) -> bool {
        truncate::needs_truncation(interner, &mut self.infer, self.max_size, subst)
    }
}

impl<I: Interner> context::InferenceTable<I, SlgContext<I>> for TruncatingInferenceTable<I> {}

impl<I: Interner> context::UnificationOps<I, SlgContext<I>> for TruncatingInferenceTable<I> {
    fn instantiate_binders_universally(&mut self, interner: &I, arg: &Binders<Goal<I>>) -> Goal<I> {
        self.infer.instantiate_binders_universally(interner, arg)
    }

    fn instantiate_binders_existentially(
        &mut self,
        interner: &I,
        arg: &Binders<Goal<I>>,
    ) -> Goal<I> {
        self.infer.instantiate_binders_existentially(interner, arg)
    }

    fn debug_ex_clause<'v>(&mut self, interner: &I, value: &'v ExClause<I>) -> Box<dyn Debug + 'v> {
        Box::new(self.infer.normalize_deep(interner, value))
    }

    fn fully_canonicalize_goal(
        &mut self,
        interner: &I,
        value: &InEnvironment<Goal<I>>,
    ) -> (UCanonical<InEnvironment<Goal<I>>>, UniverseMap) {
        let canonicalized_goal = self.infer.canonicalize(interner, value).quantified;
        let UCanonicalized {
            quantified,
            universes,
        } = self.infer.u_canonicalize(interner, &canonicalized_goal);
        (quantified, universes)
    }

    fn canonicalize_ex_clause(
        &mut self,
        interner: &I,
        value: &ExClause<I>,
    ) -> Canonical<ExClause<I>> {
        self.infer.canonicalize(interner, value).quantified
    }

    fn canonicalize_constrained_subst(
        &mut self,
        interner: &I,
        subst: Substitution<I>,
        constraints: Vec<InEnvironment<Constraint<I>>>,
    ) -> Canonical<ConstrainedSubst<I>> {
        self.infer
            .canonicalize(interner, &ConstrainedSubst { subst, constraints })
            .quantified
    }

    fn canonicalize_answer_subst(
        &mut self,
        interner: &I,
        subst: Substitution<I>,
        constraints: Vec<InEnvironment<Constraint<I>>>,
        delayed_subgoals: Vec<InEnvironment<Goal<I>>>,
    ) -> Canonical<AnswerSubst<I>> {
        self.infer
            .canonicalize(
                interner,
                &AnswerSubst {
                    subst,
                    constraints,
                    delayed_subgoals,
                },
            )
            .quantified
    }

    fn invert_goal(
        &mut self,
        interner: &I,
        value: &InEnvironment<Goal<I>>,
    ) -> Option<InEnvironment<Goal<I>>> {
        self.infer.invert(interner, value)
    }

    fn unify_generic_args_into_ex_clause(
        &mut self,
        interner: &I,
        environment: &Environment<I>,
        a: &GenericArg<I>,
        b: &GenericArg<I>,
        ex_clause: &mut ExClause<I>,
    ) -> Fallible<()> {
        let result = self.infer.unify(interner, environment, a, b)?;
        Ok(into_ex_clause(interner, result, ex_clause))
    }
}

/// Helper function
fn into_ex_clause<I: Interner>(
    interner: &I,
    result: UnificationResult<I>,
    ex_clause: &mut ExClause<I>,
) {
    ex_clause.subgoals.extend(
        result
            .goals
            .into_iter()
            .casted(interner)
            .map(Literal::Positive),
    );
}

trait SubstitutionExt<I: Interner> {
    fn may_invalidate(&self, interner: &I, subst: &Canonical<Substitution<I>>) -> bool;
}

impl<I: Interner> SubstitutionExt<I> for Substitution<I> {
    fn may_invalidate(&self, interner: &I, subst: &Canonical<Substitution<I>>) -> bool {
        self.iter(interner)
            .zip(subst.value.iter(interner))
            .any(|(new, current)| MayInvalidate { interner }.aggregate_generic_args(new, current))
    }
}

// This is a struct in case we need to add state at any point like in AntiUnifier
struct MayInvalidate<'i, I> {
    interner: &'i I,
}

impl<I: Interner> MayInvalidate<'_, I> {
    fn aggregate_generic_args(&mut self, new: &GenericArg<I>, current: &GenericArg<I>) -> bool {
        let interner = self.interner;
        match (new.data(interner), current.data(interner)) {
            (GenericArgData::Ty(ty1), GenericArgData::Ty(ty2)) => self.aggregate_tys(ty1, ty2),
            (GenericArgData::Lifetime(l1), GenericArgData::Lifetime(l2)) => {
                self.aggregate_lifetimes(l1, l2)
            }
            (GenericArgData::Const(c1), GenericArgData::Const(c2)) => self.aggregate_consts(c1, c2),
            (GenericArgData::Ty(_), _)
            | (GenericArgData::Lifetime(_), _)
            | (GenericArgData::Const(_), _) => panic!(
                "mismatched parameter kinds: new={:?} current={:?}",
                new, current
            ),
        }
    }

    /// Returns true if the two types could be unequal.
    fn aggregate_tys(&mut self, new: &Ty<I>, current: &Ty<I>) -> bool {
        let interner = self.interner;
        match (new.data(interner), current.data(interner)) {
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

            (TyData::InferenceVar(_, _), _) | (_, TyData::InferenceVar(_, _)) => {
                panic!(
                    "unexpected free inference variable in may-invalidate: {:?} vs {:?}",
                    new, current,
                );
            }

            (TyData::Apply(apply1), TyData::Apply(apply2)) => {
                self.aggregate_application_tys(apply1, apply2)
            }

            (TyData::Placeholder(p1), TyData::Placeholder(p2)) => {
                self.aggregate_placeholders(p1, p2)
            }

            (
                TyData::Alias(AliasTy::Projection(proj1)),
                TyData::Alias(AliasTy::Projection(proj2)),
            ) => self.aggregate_projection_tys(proj1, proj2),

            (
                TyData::Alias(AliasTy::Opaque(opaque_ty1)),
                TyData::Alias(AliasTy::Opaque(opaque_ty2)),
            ) => self.aggregate_opaque_ty_tys(opaque_ty1, opaque_ty2),

            // For everything else, be conservative here and just say we may invalidate.
            (TyData::Function(_), _)
            | (TyData::Dyn(_), _)
            | (TyData::Apply(_), _)
            | (TyData::Placeholder(_), _)
            | (TyData::Alias(_), _) => true,
        }
    }

    /// Returns true if the two consts could be unequal.    
    fn aggregate_lifetimes(&mut self, _: &Lifetime<I>, _: &Lifetime<I>) -> bool {
        true
    }

    /// Returns true if the two consts could be unequal.
    fn aggregate_consts(&mut self, new: &Const<I>, current: &Const<I>) -> bool {
        let interner = self.interner;
        let ConstData {
            ty: new_ty,
            value: new_value,
        } = new.data(interner);
        let ConstData {
            ty: current_ty,
            value: current_value,
        } = current.data(interner);

        if self.aggregate_tys(new_ty, current_ty) {
            return true;
        }

        match (new_value, current_value) {
            (_, ConstValue::BoundVar(_)) => {
                // see comment in aggregate_tys
                false
            }

            (ConstValue::BoundVar(_), _) => {
                // see comment in aggregate_tys
                true
            }

            (ConstValue::InferenceVar(_), _) | (_, ConstValue::InferenceVar(_)) => {
                panic!(
                    "unexpected free inference variable in may-invalidate: {:?} vs {:?}",
                    new, current,
                );
            }

            (ConstValue::Placeholder(p1), ConstValue::Placeholder(p2)) => {
                self.aggregate_placeholders(p1, p2)
            }

            (ConstValue::Concrete(c1), ConstValue::Concrete(c2)) => {
                !c1.const_eq(new_ty, c2, interner)
            }

            // Only variants left are placeholder = concrete, which always fails
            (ConstValue::Placeholder(_), _) | (ConstValue::Concrete(_), _) => true,
        }
    }

    fn aggregate_application_tys(
        &mut self,
        new: &ApplicationTy<I>,
        current: &ApplicationTy<I>,
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

    fn aggregate_placeholders(
        &mut self,
        new: &PlaceholderIndex,
        current: &PlaceholderIndex,
    ) -> bool {
        new != current
    }

    fn aggregate_projection_tys(
        &mut self,
        new: &ProjectionTy<I>,
        current: &ProjectionTy<I>,
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

    fn aggregate_opaque_ty_tys(&mut self, new: &OpaqueTy<I>, current: &OpaqueTy<I>) -> bool {
        let OpaqueTy {
            opaque_ty_id: new_name,
            substitution: new_substitution,
        } = new;
        let OpaqueTy {
            opaque_ty_id: current_name,
            substitution: current_substitution,
        } = current;

        self.aggregate_name_and_substs(
            new_name,
            new_substitution,
            current_name,
            current_substitution,
        )
    }

    fn aggregate_name_and_substs<N>(
        &mut self,
        new_name: N,
        new_substitution: &Substitution<I>,
        current_name: N,
        current_substitution: &Substitution<I>,
    ) -> bool
    where
        N: Copy + Eq + Debug,
    {
        let interner = self.interner;
        if new_name != current_name {
            return true;
        }

        let name = new_name;

        assert_eq!(
            new_substitution.len(interner),
            current_substitution.len(interner),
            "does {:?} take {} substitution or {}? can't both be right",
            name,
            new_substitution.len(interner),
            current_substitution.len(interner)
        );

        new_substitution
            .iter(interner)
            .zip(current_substitution.iter(interner))
            .any(|(new, current)| self.aggregate_generic_args(new, current))
    }
}
