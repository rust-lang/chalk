use cast::{Cast, Caster};
use ir::{DomainGoal, LeafGoal, Goal, QuantifierKind, InEnvironment, Substitution};
use solve::infer::unify::UnificationResult;
use solve::slg::{ExClause, Literal, Satisfiable};
use solve::slg::context::prelude::*;

/// Simplifies an HH goal into a series of positive domain goals
/// and negative HH goals. This operation may fail if the HH goal
/// includes unifications that cannot be completed.
pub(super) fn simplify_hh_goal<C: Context>(
    infer: &mut C::InferenceTable,
    subst: Substitution,
    initial_goal: InEnvironment<Goal<DomainGoal>>,
) -> Satisfiable<ExClause> {
    let mut ex_clause = ExClause {
        subst,
        delayed_literals: vec![],
        constraints: vec![],
        subgoals: vec![],
    };

    // A stack of higher-level goals to process.
    let mut pending_goals = vec![initial_goal];

    while let Some(InEnvironment { environment, goal }) = pending_goals.pop() {
        match goal {
            Goal::Quantified(QuantifierKind::ForAll, subgoal) => {
                let subgoal = infer.instantiate_binders_universally(&subgoal);
                pending_goals.push(InEnvironment::new(&environment, *subgoal));
            }
            Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                let subgoal = infer.instantiate_binders_existentially(&subgoal);
                pending_goals.push(InEnvironment::new(&environment, *subgoal))
            }
            Goal::Implies(wc, subgoal) => {
                let new_environment = &environment.add_clauses(wc);
                pending_goals.push(InEnvironment::new(&new_environment, *subgoal));
            }
            Goal::And(subgoal1, subgoal2) => {
                pending_goals.push(InEnvironment::new(&environment, *subgoal1));
                pending_goals.push(InEnvironment::new(&environment, *subgoal2));
            }
            Goal::Not(subgoal) => {
                let subgoal = (*subgoal).clone();
                ex_clause
                    .subgoals
                    .push(Literal::Negative(InEnvironment::new(&environment, subgoal)));
            }
            Goal::Leaf(LeafGoal::EqGoal(ref eq_goal)) => {
                let UnificationResult { goals, constraints } = {
                    match infer.unify(&environment, &eq_goal.a, &eq_goal.b) {
                        Ok(v) => v,
                        Err(_) => return Satisfiable::No,
                    }
                };

                ex_clause.constraints.extend(constraints);
                ex_clause
                    .subgoals
                    .extend(goals.into_iter().casted().map(Literal::Positive));
            }
            Goal::Leaf(LeafGoal::DomainGoal(domain_goal)) => {
                let domain_goal = domain_goal.cast();
                ex_clause
                    .subgoals
                    .push(Literal::Positive(InEnvironment::new(
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
                    .push(Literal::Negative(InEnvironment::new(&environment, goal)));
            }
        }
    }

    Satisfiable::Yes(ex_clause)
}
