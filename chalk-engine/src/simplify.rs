use crate::context::{Context, ContextOps, InferenceTable};
use crate::fallible::Fallible;
use crate::forest::Forest;
use crate::hh::HhGoal;
use crate::{ExClause, Literal, TimeStamp};

impl<C: Context> Forest<C> {
    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_hh_goal(
        context: &impl ContextOps<C>,
        infer: &mut dyn InferenceTable<C>,
        subst: C::Substitution,
        initial_environment: C::Environment,
        initial_hh_goal: HhGoal<C>,
    ) -> Fallible<ExClause<C>> {
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
        let mut pending_goals = vec![(initial_environment, initial_hh_goal)];

        while let Some((environment, hh_goal)) = pending_goals.pop() {
            match hh_goal {
                HhGoal::ForAll(subgoal) => {
                    let subgoal =
                        infer.instantiate_binders_universally(context.interner(), &subgoal);
                    pending_goals.push((environment, context.into_hh_goal(subgoal)));
                }
                HhGoal::Exists(subgoal) => {
                    let subgoal =
                        infer.instantiate_binders_existentially(context.interner(), &subgoal);
                    pending_goals.push((environment, context.into_hh_goal(subgoal)))
                }
                HhGoal::Implies(wc, subgoal) => {
                    let new_environment = context.add_clauses(&environment, wc);
                    pending_goals.push((new_environment, context.into_hh_goal(subgoal)));
                }
                HhGoal::All(subgoals) => {
                    for subgoal in subgoals {
                        pending_goals.push((environment.clone(), context.into_hh_goal(subgoal)));
                    }
                }
                HhGoal::Not(subgoal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(C::goal_in_environment(
                            &environment,
                            subgoal,
                        )));
                }
                HhGoal::Unify(variance, a, b) => infer.unify_parameters_into_ex_clause(
                    context.interner(),
                    &environment,
                    variance,
                    &a,
                    &b,
                    &mut ex_clause,
                )?,
                HhGoal::DomainGoal(domain_goal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Positive(C::goal_in_environment(
                            &environment,
                            context.into_goal(domain_goal),
                        )));
                }
                HhGoal::CannotProve => {
                    ex_clause.ambiguous = true;
                }
            }
        }

        Ok(ex_clause)
    }
}
