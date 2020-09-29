use crate::context::{AnswerResult, AnswerStream};
use crate::forest::Forest;
use crate::slg::aggregate::AggregateOps;
use crate::slg::SlgContextOps;
use chalk_ir::interner::Interner;
use chalk_ir::{Canonical, ConstrainedSubst, Goal, InEnvironment, UCanonical};
use chalk_solve::{RustIrDatabase, Solution, Solver, SubstitutionResult};

use std::fmt;

pub struct SLGSolver<I: Interner> {
    pub(crate) forest: Forest<I>,
    pub(crate) max_size: usize,
    pub(crate) expected_answers: Option<usize>,
}

impl<I: Interner> SLGSolver<I> {
    pub fn new(max_size: usize, expected_answers: Option<usize>) -> Self {
        Self {
            forest: Forest::new(),
            max_size,
            expected_answers,
        }
    }
}

impl<I: Interner> fmt::Debug for SLGSolver<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "SLGSolver")
    }
}

impl<I: Interner> Solver<I> for SLGSolver<I> {
    fn solve(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> Option<Solution<I>> {
        let ops = SlgContextOps::new(program, self.max_size, self.expected_answers);
        ops.make_solution(goal, self.forest.iter_answers(&ops, goal), || true)
    }

    fn solve_limited(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        should_continue: &dyn std::ops::Fn() -> bool,
    ) -> Option<Solution<I>> {
        let ops = SlgContextOps::new(program, self.max_size, self.expected_answers);
        ops.make_solution(goal, self.forest.iter_answers(&ops, goal), should_continue)
    }

    fn solve_multiple(
        &mut self,
        program: &dyn RustIrDatabase<I>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        f: &mut dyn FnMut(SubstitutionResult<Canonical<ConstrainedSubst<I>>>, bool) -> bool,
    ) -> bool {
        let ops = SlgContextOps::new(program, self.max_size, self.expected_answers);
        let mut answers = self.forest.iter_answers(&ops, goal);
        loop {
            let subst = match answers.next_answer(|| true) {
                AnswerResult::Answer(answer) => {
                    if !answer.ambiguous {
                        SubstitutionResult::Definite(answer.subst)
                    } else if answer
                        .subst
                        .value
                        .subst
                        .is_identity_subst(ops.program().interner())
                    {
                        SubstitutionResult::Floundered
                    } else {
                        SubstitutionResult::Ambiguous(answer.subst)
                    }
                }
                AnswerResult::Floundered => SubstitutionResult::Floundered,
                AnswerResult::NoMoreSolutions => {
                    return true;
                }
                AnswerResult::QuantumExceeded => continue,
            };

            if !f(subst, !answers.peek_answer(|| true).is_no_more_solutions()) {
                return false;
            }
        }
    }
}
