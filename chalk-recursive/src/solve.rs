use super::combine;
use super::fulfill::Fulfill;
use crate::{Minimums, UCanonicalGoal};
use chalk_ir::fold::Fold;
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::{
    Binders, Canonical, ClausePriority, DomainGoal, Fallible, Floundered, Goal, GoalData,
    InEnvironment, NoSolution, ProgramClause, ProgramClauseData, ProgramClauseImplication,
    Substitution, UCanonical,
};
use chalk_solve::clauses::program_clauses_for_goal;
use chalk_solve::debug_span;
use chalk_solve::infer::InferenceTable;
use chalk_solve::{Guidance, RustIrDatabase, Solution};
use tracing::{debug, instrument};

pub(super) trait SolveDatabase<I: Interner>: Sized {
    fn solve_goal(
        &mut self,
        goal: UCanonical<InEnvironment<Goal<I>>>,
        minimums: &mut Minimums,
    ) -> Fallible<Solution<I>>;

    fn max_size(&self) -> usize;

    fn interner(&self) -> &I;

    fn db(&self) -> &dyn RustIrDatabase<I>;
}

/// The `solve_iteration` method -- implemented for any type that implements
/// `SolveDb`.
pub(super) trait SolveIteration<I: Interner>: SolveDatabase<I> {
    /// Executes one iteration of the recursive solver, computing the current
    /// solution to the given canonical goal. This is used as part of a loop in
    /// the case of cyclic goals.
    #[instrument(level = "debug", skip(self))]
    fn solve_iteration(
        &mut self,
        canonical_goal: &UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        let UCanonical {
            universes,
            canonical:
                Canonical {
                    binders,
                    value: InEnvironment { environment, goal },
                },
        } = canonical_goal.clone();

        match goal.data(self.interner()) {
            GoalData::DomainGoal(domain_goal) => {
                let canonical_goal = UCanonical {
                    universes,
                    canonical: Canonical {
                        binders,
                        value: InEnvironment {
                            environment,
                            goal: domain_goal.clone(),
                        },
                    },
                };

                // "Domain" goals (i.e., leaf goals that are Rust-specific) are
                // always solved via some form of implication. We can either
                // apply assumptions from our environment (i.e. where clauses),
                // or from the lowered program, which includes fallback
                // clauses. We try each approach in turn:

                let (prog_solution, prog_prio) = {
                    debug_span!("prog_clauses");

                    let prog_clauses = self.program_clauses_for_goal(&canonical_goal);
                    match prog_clauses {
                        Ok(clauses) => self.solve_from_clauses(&canonical_goal, clauses, minimums),
                        Err(Floundered) => {
                            (Ok(Solution::Ambig(Guidance::Unknown)), ClausePriority::High)
                        }
                    }
                };
                debug!(?prog_solution);

                (prog_solution, prog_prio)
            }

            _ => {
                let canonical_goal = UCanonical {
                    universes,
                    canonical: Canonical {
                        binders,
                        value: InEnvironment { environment, goal },
                    },
                };

                self.solve_via_simplification(&canonical_goal, minimums)
            }
        }
    }
}

impl<S, I> SolveIteration<I> for S
where
    S: SolveDatabase<I>,
    I: Interner,
{
}

/// Helper methods for `solve_iteration`, private to this module.
trait SolveIterationHelpers<I: Interner>: SolveDatabase<I> {
    #[instrument(level = "debug", skip(self, minimums))]
    fn solve_via_simplification(
        &mut self,
        canonical_goal: &UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        let (infer, subst, goal) = self.new_inference_table(canonical_goal);
        match Fulfill::new_with_simplification(self, infer, subst, goal) {
            Ok(fulfill) => (fulfill.solve(minimums), ClausePriority::High),
            Err(e) => (Err(e), ClausePriority::High),
        }
    }

    /// See whether we can solve a goal by implication on any of the given
    /// clauses. If multiple such solutions are possible, we attempt to combine
    /// them.
    fn solve_from_clauses<C>(
        &mut self,
        canonical_goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
        clauses: C,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority)
    where
        C: IntoIterator<Item = ProgramClause<I>>,
    {
        let mut cur_solution = None;
        for program_clause in clauses {
            debug_span!("solve_from_clauses", clause = ?program_clause);

            // If we have a completely ambiguous answer, it's not going to get better, so stop
            if cur_solution == Some((Solution::Ambig(Guidance::Unknown), ClausePriority::High)) {
                return (Ok(Solution::Ambig(Guidance::Unknown)), ClausePriority::High);
            }

            let ProgramClauseData(implication) = program_clause.data(self.interner());
            let res = self.solve_via_implication(canonical_goal, implication, minimums);

            if let (Ok(solution), priority) = res {
                debug!(?solution, ?priority, "Ok");
                cur_solution = Some(match cur_solution {
                    None => (solution, priority),
                    Some((cur, cur_priority)) => combine::with_priorities(
                        self.interner(),
                        &canonical_goal.canonical.value.goal,
                        cur,
                        cur_priority,
                        solution,
                        priority,
                    ),
                });
            } else {
                debug!("Error");
            }
        }
        cur_solution.map_or((Err(NoSolution), ClausePriority::High), |(s, p)| (Ok(s), p))
    }

    /// Modus ponens! That is: try to apply an implication by proving its premises.
    #[instrument(level = "info", skip(self, minimums))]
    fn solve_via_implication(
        &mut self,
        canonical_goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
        clause: &Binders<ProgramClauseImplication<I>>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        let (infer, subst, goal) = self.new_inference_table(canonical_goal);
        let clause = subst.apply(clause.clone(), self.interner());
        match Fulfill::new_with_clause(self, infer, subst, goal, &clause) {
            Ok(fulfill) => (fulfill.solve(minimums), clause.skip_binders().priority),
            Err(e) => (Err(e), ClausePriority::High),
        }
    }

    fn new_inference_table<T: Fold<I, Result = T> + HasInterner<Interner = I> + Clone>(
        &self,
        ucanonical_goal: &UCanonical<InEnvironment<T>>,
    ) -> (InferenceTable<I>, Substitution<I>, InEnvironment<T::Result>) {
        let (infer, subst, canonical_goal) = InferenceTable::from_canonical(
            self.interner(),
            ucanonical_goal.universes,
            ucanonical_goal.canonical.clone(),
        );
        (infer, subst, canonical_goal)
    }

    fn program_clauses_for_goal(
        &self,
        canonical_goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
    ) -> Result<Vec<ProgramClause<I>>, Floundered> {
        program_clauses_for_goal(self.db(), &canonical_goal)
    }
}

impl<S, I> SolveIterationHelpers<I> for S
where
    S: SolveDatabase<I>,
    I: Interner,
{
}
