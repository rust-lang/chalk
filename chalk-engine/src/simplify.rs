use fallible::Fallible;
use {ExClause, Literal};
use forest::Forest;
use hh::HhGoal;
use context::prelude::*;

impl<C: Context, CO: ContextOps<C>> Forest<C, CO> {
    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_hh_goal<I: Context>(
        infer: &mut dyn InferenceTable<C, I>,
        subst: I::Substitution,
        initial_environment: &I::Environment,
        initial_hh_goal: HhGoal<I>,
    ) -> Fallible<ExClause<I>> {
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
                    pending_goals.push((environment, infer.into_hh_goal(subgoal)));
                }
                HhGoal::Exists(subgoal) => {
                    let subgoal = infer.instantiate_binders_existentially(&subgoal);
                    pending_goals.push((environment, infer.into_hh_goal(subgoal)))
                }
                HhGoal::Implies(wc, subgoal) => {
                    let new_environment = infer.add_clauses(&environment, wc);
                    pending_goals.push((new_environment, infer.into_hh_goal(subgoal)));
                }
                HhGoal::And(subgoal1, subgoal2) => {
                    pending_goals.push((environment.clone(), infer.into_hh_goal(subgoal1)));
                    pending_goals.push((environment, infer.into_hh_goal(subgoal2)));
                }
                HhGoal::Not(subgoal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(I::goal_in_environment(&environment, subgoal)));
                }
                HhGoal::Unify(variance, a, b) => {
                    let result = infer.unify_parameters(&environment, variance, &a, &b)?;
                    infer.into_ex_clause(result, &mut ex_clause)
                }
                HhGoal::DomainGoal(domain_goal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Positive(I::goal_in_environment(
                            &environment,
                            infer.into_goal(domain_goal),
                        )));
                }
                HhGoal::CannotProve => {
                    // You can think of `CannotProve` as a special
                    // goal that is only provable if `not {
                    // CannotProve }`. Trying to prove this, of
                    // course, will always create a negative cycle and
                    // hence a delayed literal that cannot be
                    // resolved.
                    let goal = infer.cannot_prove();
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(I::goal_in_environment(&environment, goal)));
                }
            }
        }

        Ok(ex_clause)
    }
}
