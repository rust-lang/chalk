use super::combine;
use super::fulfill::{Fulfill, RecursiveInferenceTable, RecursiveSolver};
use super::lib::{Guidance, Minimums, Solution, UCanonicalGoal};
use super::search_graph::{DepthFirstNumber, SearchGraph};
use super::stack::{Stack, StackDepth};
use super::Solver;
use crate::clauses::program_clauses_for_goal;
use crate::infer::{InferenceTable, ParameterEnaVariableExt};
use crate::solve::truncate;
use crate::{coinductive_goal::IsCoinductive, RustIrDatabase};
use chalk_ir::fold::Fold;
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::visit::Visit;
use chalk_ir::zip::Zip;
use chalk_ir::{debug, debug_heading, info, info_heading};
use chalk_ir::{
    Binders, Canonical, ClausePriority, ConstrainedSubst, Constraint, DomainGoal, Environment,
    Fallible, Floundered, GenericArg, Goal, GoalData, InEnvironment, NoSolution, ProgramClause,
    ProgramClauseData, ProgramClauseImplication, Substitution, UCanonical, UniverseMap,
    VariableKinds,
};
use rustc_hash::FxHashMap;
use std::fmt::Debug;

impl<'me, I: Interner> Solver<'me, I> {
    pub(super) fn solve_iteration(
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

        match goal.data(self.program.interner()) {
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

                let InEnvironment { environment, goal } = &canonical_goal.canonical.value;

                let (prog_solution, prog_prio) = {
                    debug_heading!("prog_clauses");

                    let prog_clauses = self.program_clauses_for_goal(environment, &goal);
                    match prog_clauses {
                        Ok(clauses) => self.solve_from_clauses(&canonical_goal, clauses, minimums),
                        Err(Floundered) => {
                            (Ok(Solution::Ambig(Guidance::Unknown)), ClausePriority::High)
                        }
                    }
                };
                debug!("prog_solution={:?}", prog_solution);

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

    fn solve_via_simplification(
        &mut self,
        canonical_goal: &UCanonicalGoal<I>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        debug_heading!("solve_via_simplification({:?})", canonical_goal);
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
            debug_heading!("clause={:?}", program_clause);

            // If we have a completely ambiguous answer, it's not going to get better, so stop
            if cur_solution == Some((Solution::Ambig(Guidance::Unknown), ClausePriority::High)) {
                return (Ok(Solution::Ambig(Guidance::Unknown)), ClausePriority::High);
            }

            let res = match program_clause.data(self.program.interner()) {
                ProgramClauseData::Implies(implication) => self.solve_via_implication(
                    canonical_goal,
                    &Binders::new(
                        VariableKinds::from(self.program.interner(), vec![]),
                        implication.clone(),
                    ),
                    minimums,
                ),
                ProgramClauseData::ForAll(implication) => {
                    self.solve_via_implication(canonical_goal, implication, minimums)
                }
            };
            if let (Ok(solution), priority) = res {
                debug!("ok: solution={:?} prio={:?}", solution, priority);
                cur_solution = Some(match cur_solution {
                    None => (solution, priority),
                    Some((cur, cur_priority)) => combine::with_priorities(
                        self.program.interner(),
                        &canonical_goal.canonical.value.goal,
                        cur,
                        cur_priority,
                        solution,
                        priority,
                    ),
                });
            } else {
                debug!("error");
            }
        }
        cur_solution.map_or((Err(NoSolution), ClausePriority::High), |(s, p)| (Ok(s), p))
    }

    /// Modus ponens! That is: try to apply an implication by proving its premises.
    fn solve_via_implication(
        &mut self,
        canonical_goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
        clause: &Binders<ProgramClauseImplication<I>>,
        minimums: &mut Minimums,
    ) -> (Fallible<Solution<I>>, ClausePriority) {
        info_heading!(
            "solve_via_implication(\
         \n    canonical_goal={:?},\
         \n    clause={:?})",
            canonical_goal,
            clause
        );

        let (infer, subst, goal) = self.new_inference_table(canonical_goal);
        match Fulfill::new_with_clause(self, infer, subst, goal, clause) {
            Ok(fulfill) => (fulfill.solve(minimums), clause.skip_binders().priority),
            Err(e) => (Err(e), ClausePriority::High),
        }
    }
}
