use fallible::NoSolution;
use solve::slg::{ExClause, Literal, Satisfiable};
use solve::slg::forest::Forest;
use solve::slg::hh::HhGoal;
use solve::slg::context::prelude::*;

impl<C: Context> Forest<C> {
    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_hh_goal(
        infer: &mut C::InferenceTable,
        subst: C::Substitution,
        initial_environment: &C::Environment,
        initial_hh_goal: HhGoal<C>,
    ) -> Satisfiable<ExClause<C>> {
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
                        .push(Literal::Negative(C::goal_in_environment(&environment, subgoal)));
                }
                HhGoal::Unify(a, b) => {
                    match infer.unify_parameters(&environment, &a, &b) {
                        Ok(result) => result.into_ex_clause(&mut ex_clause),
                        Err(NoSolution) => return Satisfiable::No,
                    }
                }
                HhGoal::DomainGoal(domain_goal) => {
                    ex_clause
                        .subgoals
                        .push(Literal::Positive(C::goal_in_environment(
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
                    let goal = C::Goal::cannot_prove();
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(C::goal_in_environment(&environment, goal)));
                }
            }
        }

        Satisfiable::Yes(ex_clause)
    }
}
