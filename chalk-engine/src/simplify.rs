use crate::context::{Context, ContextOps, InferenceTable};
use crate::forest::Forest;
use crate::hh::HhGoal;
use crate::{ExClause, Literal, TimeStamp};
use chalk_base::results::Fallible;

use chalk_ir::interner::Interner;
use chalk_ir::{Environment, InEnvironment, Substitution};

impl<I: Interner, C: Context<I>> Forest<I, C> {
    /// Simplifies an HH goal into a series of positive domain goals
    /// and negative HH goals. This operation may fail if the HH goal
    /// includes unifications that cannot be completed.
    pub(super) fn simplify_hh_goal(
        context: &impl ContextOps<I, C>,
        infer: &mut dyn InferenceTable<I, C>,
        subst: Substitution<I>,
        initial_environment: Environment<I>,
        initial_hh_goal: HhGoal<I, C>,
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
                        .push(Literal::Negative(InEnvironment::new(&environment, subgoal)));
                }
                HhGoal::Unify(variance, a, b) => infer.unify_generic_args_into_ex_clause(
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
                        .push(Literal::Positive(InEnvironment::new(&environment, context.into_goal(domain_goal))));
                }
                HhGoal::CannotProve => {
                    ex_clause.ambiguous = true;
                }
            }
        }

        Ok(ex_clause)
    }
}
