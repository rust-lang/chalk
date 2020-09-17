use crate::forest::Forest;
use crate::slg::{SlgContextOps, TruncatingInferenceTable, UnificationOps};
use crate::{ExClause, Literal, TimeStamp};

use chalk_ir::cast::Cast;
use chalk_ir::interner::Interner;
use chalk_ir::{
    Environment, Fallible, Goal, GoalData, InEnvironment, QuantifierKind, Substitution,
};
use tracing::debug;

impl<I: Interner> Forest<I> {
    /// Simplifies a goal into a series of positive domain goals
    /// and negative goals. This operation may fail if the goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_goal(
        context: &SlgContextOps<I>,
        infer: &mut TruncatingInferenceTable<I>,
        subst: Substitution<I>,
        initial_environment: Environment<I>,
        initial_goal: Goal<I>,
    ) -> Fallible<ExClause<I>> {
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
                    let subgoal = infer
                        .instantiate_binders_universally(context.program().interner(), &subgoal);
                    pending_goals.push((environment, subgoal.clone()));
                }
                GoalData::Quantified(QuantifierKind::Exists, subgoal) => {
                    let subgoal = infer
                        .instantiate_binders_existentially(context.program().interner(), &subgoal);
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
                GoalData::EqGoal(goal) => infer.unify_generic_args_into_ex_clause(
                    context.program().interner(),
                    &environment,
                    &goal.a,
                    &goal.b,
                    &mut ex_clause,
                )?,
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

        Ok(ex_clause)
    }
}
