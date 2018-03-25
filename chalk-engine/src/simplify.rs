use crate::fallible::Fallible;
use crate::{ExClause, Literal};
use crate::forest::Forest;
use crate::hh::HhGoal;
use crate::context::prelude::*;

impl<C: Context> Forest<C> {
    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_hh_goal<I: InferenceContext<C>>(
        infer: &mut dyn InferenceTable<C, I>,
        subst: I::Substitution,
        initial_environment: &I::Environment,
        initial_hh_goal: HhGoal<C, I>,
    ) -> Fallible<ExClause<C, I>> {
        let mut ex_clause = ExClause {
            subst,
            delayed_literals: vec![],
            constraints: vec![],
            subgoals: vec![],
        };

        // A stack of higher-level goals to process.
        let mut pending_goals = vec![(initial_environment.clone(), initial_hh_goal)];

        while let Some((environment, hh_goal)) = pending_goals.pop() {
            match hh_goal {
                HhGoal::ForAll(subgoal) => {
                    let subgoal = infer.instantiate_binders_universally(&subgoal);
                    pending_goals.push((environment, subgoal.into_hh_goal()));
                }
                HhGoal::Exists(subgoal) => {
                    let subgoal = infer.instantiate_binders_existentially(&subgoal);
                    pending_goals.push((environment, subgoal.into_hh_goal()))
                }
                HhGoal::Implies(wc, subgoal) => {
                    let new_environment = environment.add_clauses(wc);
                    pending_goals.push((new_environment, subgoal.into_hh_goal()));
                }
                HhGoal::And(subgoal1, subgoal2) => {
                    pending_goals.push((environment.clone(), subgoal1.into_hh_goal()));
                    pending_goals.push((environment, subgoal2.into_hh_goal()));
                }
                HhGoal::Not(subgoal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(I::GoalInEnvironment::new(&environment, subgoal)));
                }
                HhGoal::Unify(a, b) => {
                    infer.unify_parameters(&environment, &a, &b)?
                        .into_ex_clause(&mut ex_clause)
                }
                HhGoal::DomainGoal(domain_goal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Positive(I::GoalInEnvironment::new(
                            &environment,
                            domain_goal.into_goal(),
                        )));
                }
                HhGoal::CannotProve => {
                    // You can think of `CannotProve` as a special
                    // goal that is only provable if `not {
                    // CannotProve }`. Trying to prove this, of
                    // course, will always create a negative cycle and
                    // hence a delayed literal that cannot be
                    // resolved.
                    let goal = I::Goal::cannot_prove();
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(I::GoalInEnvironment::new(&environment, goal)));
                }
            }
        }

        Ok(ex_clause)
    }
}
