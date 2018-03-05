use cast::Cast;
use fallible::NoSolution;
use ir::{DomainGoal, Goal, LeafGoal, QuantifierKind, Substitution};
use solve::slg::{ExClause, Literal, Satisfiable};
use solve::slg::forest::Forest;
use solve::slg::context::prelude::*;

impl<C: Context> Forest<C> {
    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_hh_goal(
        infer: &mut C::InferenceTable,
        subst: Substitution,
        initial_environment: &C::Environment,
        initial_goal: Goal<DomainGoal>,
    ) -> Satisfiable<ExClause<C>> {
        let mut ex_clause = ExClause {
            subst,
            delayed_literals: vec![],
            constraints: vec![],
            subgoals: vec![],
        };

        // A stack of higher-level goals to process.
        let mut pending_goals = vec![(initial_environment.clone(), initial_goal)];

        while let Some((environment, goal)) = pending_goals.pop() {
            match goal {
                Goal::Quantified(QuantifierKind::ForAll, subgoal) => {
                    let subgoal = infer.instantiate_binders_universally(&subgoal);
                    pending_goals.push((environment, *subgoal));
                }
                Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                    let subgoal = infer.instantiate_binders_existentially(&subgoal);
                    pending_goals.push((environment, *subgoal))
                }
                Goal::Implies(wc, subgoal) => {
                    let new_environment = environment.add_clauses(wc);
                    pending_goals.push((new_environment, *subgoal));
                }
                Goal::And(subgoal1, subgoal2) => {
                    pending_goals.push((environment.clone(), *subgoal1));
                    pending_goals.push((environment, *subgoal2));
                }
                Goal::Not(subgoal) => {
                    let subgoal = (*subgoal).clone();
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(C::goal_in_environment(&environment, subgoal)));
                }
                Goal::Leaf(LeafGoal::EqGoal(ref eq_goal)) => {
                    match infer.unify_parameters(&environment, &eq_goal.a, &eq_goal.b) {
                        Ok(result) => result.into_ex_clause(&mut ex_clause),
                        Err(NoSolution) => return Satisfiable::No,
                    }
                }
                Goal::Leaf(LeafGoal::DomainGoal(domain_goal)) => {
                    let domain_goal = domain_goal.cast();
                    ex_clause
                        .subgoals
                        .push(Literal::Positive(C::goal_in_environment(
                            &environment,
                            domain_goal,
                        )));
                }
                Goal::CannotProve(()) => {
                    // You can think of `CannotProve` as a special
                    // goal that is only provable if `not {
                    // CannotProve }`. Trying to prove this, of
                    // course, will always create a negative cycle and
                    // hence a delayed literal that cannot be
                    // resolved.
                    ex_clause
                        .subgoals
                        .push(Literal::Negative(C::goal_in_environment(&environment, goal)));
                }
            }
        }

        Satisfiable::Yes(ex_clause)
    }
}
