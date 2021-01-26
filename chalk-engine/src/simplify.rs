use crate::forest::Forest;
use crate::slg::SlgContextOps;
use crate::{ExClause, Literal, TimeStamp};

use chalk_ir::cast::{Cast, Caster};
use chalk_ir::interner::Interner;
use chalk_ir::{
    Environment, FallibleOrFloundered, Goal, GoalData, InEnvironment, QuantifierKind, Substitution,
    TyKind, TyVariableKind, Variance,
};
use chalk_solve::infer::InferenceTable;
use tracing::debug;

impl<I: Interner> Forest<I> {
    /// Simplifies a goal into a series of positive domain goals
    /// and negative goals. This operation may fail if the goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_goal(
        context: &SlgContextOps<I>,
        infer: &mut InferenceTable<I>,
        subst: Substitution<I>,
        initial_environment: Environment<I>,
        initial_goal: Goal<I>,
    ) -> FallibleOrFloundered<ExClause<I>> {
        let mut ex_clause = ExClause {
            subst,
            ambiguous: false,
            constraints: vec![],
            subgoals: vec![],
            delayed_subgoals: vec![],
            answer_time: TimeStamp::default(),
            floundered_subgoals: vec![],
        };

        // A stack of higher-level goals to process.
        let mut pending_goals = vec![(initial_environment, initial_goal)];

        while let Some((environment, goal)) = pending_goals.pop() {
            match goal.data(context.program().interner()) {
                GoalData::Quantified(QuantifierKind::ForAll, subgoal) => {
                    let subgoal = infer.instantiate_binders_universally(
                        context.program().interner(),
                        subgoal.clone(),
                    );
                    pending_goals.push((environment, subgoal.clone()));
                }
                GoalData::Quantified(QuantifierKind::Exists, subgoal) => {
                    let subgoal = infer.instantiate_binders_existentially(
                        context.program().interner(),
                        subgoal.clone(),
                    );
                    pending_goals.push((environment, subgoal.clone()));
                }
                GoalData::Implies(wc, subgoal) => {
                    let new_environment = environment.add_clauses(
                        context.program().interner(),
                        wc.iter(context.program().interner()).cloned(),
                    );
                    pending_goals.push((new_environment, subgoal.clone()));
                }
                GoalData::All(subgoals) => {
                    for subgoal in subgoals.iter(context.program().interner()) {
                        pending_goals.push((environment.clone(), subgoal.clone()));
                    }
                }
                GoalData::Not(subgoal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(InEnvironment::new(
                            &environment,
                            subgoal.clone(),
                        )));
                }
                GoalData::EqGoal(goal) => {
                    let interner = context.program().interner();
                    let db = context.unification_database();
                    let a = &goal.a;
                    let b = &goal.b;

                    let result =
                        match infer.relate(interner, db, &environment, Variance::Invariant, a, b) {
                            Ok(r) => r,
                            Err(_) => return FallibleOrFloundered::NoSolution,
                        };
                    ex_clause.subgoals.extend(
                        result
                            .goals
                            .into_iter()
                            .casted(interner)
                            .map(Literal::Positive),
                    );
                }
                GoalData::SubtypeGoal(goal) => {
                    let interner = context.program().interner();
                    let db = context.unification_database();
                    let a_norm = infer.normalize_ty_shallow(interner, &goal.a);
                    let a = a_norm.as_ref().unwrap_or(&goal.a);
                    let b_norm = infer.normalize_ty_shallow(interner, &goal.b);
                    let b = b_norm.as_ref().unwrap_or(&goal.b);

                    if matches!(
                        a.kind(interner),
                        TyKind::InferenceVar(_, TyVariableKind::General)
                    ) && matches!(
                        b.kind(interner),
                        TyKind::InferenceVar(_, TyVariableKind::General)
                    ) {
                        return FallibleOrFloundered::Floundered;
                    }

                    let result =
                        match infer.relate(interner, db, &environment, Variance::Covariant, a, b) {
                            Ok(r) => r,
                            Err(_) => return FallibleOrFloundered::Floundered,
                        };
                    ex_clause.subgoals.extend(
                        result
                            .goals
                            .into_iter()
                            .casted(interner)
                            .map(Literal::Positive),
                    );
                }
                GoalData::DomainGoal(domain_goal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Positive(InEnvironment::new(
                            &environment,
                            domain_goal.clone().cast(context.program().interner()),
                        )));
                }
                GoalData::CannotProve => {
                    debug!("Marking Strand as ambiguous because of a `CannotProve` subgoal");
                    ex_clause.ambiguous = true;
                }
            }
        }

        FallibleOrFloundered::Ok(ex_clause)
    }
}
