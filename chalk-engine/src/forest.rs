use crate::context::{AnswerResult, AnswerStream, Context, ContextOps};
use crate::logic::RootSearchFail;
use crate::table::AnswerIndex;
use crate::tables::Tables;
use crate::{TableIndex, TimeStamp};
use std::fmt::Display;

use chalk_ir::interner::Interner;
use chalk_ir::{Canonical, ConstrainedSubst, Goal, InEnvironment, Substitution, UCanonical};

pub struct Forest<I: Interner, C: Context<I>> {
    context: C,
    pub(crate) tables: Tables<I>,

    /// This is a clock which always increases. It is
    /// incremented every time a new subgoal is followed.
    /// This effectively gives us way to track what depth
    /// and loop a table or strand was last followed.
    pub(crate) clock: TimeStamp,
}

impl<I: Interner, C: Context<I>> Forest<I, C> {
    pub fn new(context: C) -> Self {
        Forest {
            context,
            tables: Tables::new(),
            clock: TimeStamp::default(),
        }
    }

    /// Gives access to `self.context`. In fact, the SLG solver
    /// doesn't ever use `self.context` for anything, and only cares
    /// about the associated types and methods defined on it.  But the
    /// creator of the forest can use the context field to store
    /// configuration info (e.g., in chalk, we store the max size of a
    /// term in here).
    pub fn context(&self) -> &C {
        &self.context
    }

    // Gets the next clock TimeStamp. This will never decrease.
    pub(crate) fn increment_clock(&mut self) -> TimeStamp {
        self.clock.increment();
        self.clock
    }

    /// Returns a "solver" for a given goal in the form of an
    /// iterator. Each time you invoke `next`, it will do the work to
    /// extract one more answer. These answers are cached in between
    /// invocations. Invoking `next` fewer times is preferable =)
    fn iter_answers<'f>(
        &'f mut self,
        context: &'f impl ContextOps<I, C>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> impl AnswerStream<I, C> + 'f {
        let table = self.get_or_create_table_for_ucanonical_goal(context, goal.clone());
        let answer = AnswerIndex::ZERO;
        ForestSolver {
            forest: self,
            context,
            table,
            answer,
        }
    }

    /// Solves a given goal, producing the solution. This will do only
    /// as much work towards `goal` as it has to (and that works is
    /// cached for future attempts).
    pub fn solve(
        &mut self,
        context: &impl ContextOps<I, C>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        should_continue: impl Fn() -> bool,
    ) -> Option<C::Solution> {
        context.make_solution(&goal, self.iter_answers(context, goal), should_continue)
    }

    /// Solves a given goal, producing the solution. This will do only
    /// as much work towards `goal` as it has to (and that works is
    /// cached for future attempts). Calls provided function `f` to
    /// iterate over multiple solutions until the function return `false`.
    pub fn solve_multiple(
        &mut self,
        context: &impl ContextOps<I, C>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
        mut f: impl FnMut(SubstitutionResult<Canonical<ConstrainedSubst<I>>>, bool) -> bool,
    ) -> bool {
        let mut answers = self.iter_answers(context, goal);
        loop {
            let subst = match answers.next_answer(|| true) {
                AnswerResult::Answer(answer) => {
                    if !answer.ambiguous {
                        SubstitutionResult::Definite(answer.subst)
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

#[derive(Debug)]
pub enum SubstitutionResult<S> {
    Definite(S),
    Ambiguous(S),
    Floundered,
}

impl<S> SubstitutionResult<S> {
    pub fn as_ref(&self) -> SubstitutionResult<&S> {
        match self {
            SubstitutionResult::Definite(subst) => SubstitutionResult::Definite(subst),
            SubstitutionResult::Ambiguous(subst) => SubstitutionResult::Ambiguous(subst),
            SubstitutionResult::Floundered => SubstitutionResult::Floundered,
        }
    }
    pub fn map<U, F: FnOnce(S) -> U>(self, f: F) -> SubstitutionResult<U> {
        match self {
            SubstitutionResult::Definite(subst) => SubstitutionResult::Definite(f(subst)),
            SubstitutionResult::Ambiguous(subst) => SubstitutionResult::Ambiguous(f(subst)),
            SubstitutionResult::Floundered => SubstitutionResult::Floundered,
        }
    }
}

impl<S: Display> Display for SubstitutionResult<S> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubstitutionResult::Definite(subst) => write!(fmt, "{}", subst),
            SubstitutionResult::Ambiguous(subst) => write!(fmt, "Ambiguous({})", subst),
            SubstitutionResult::Floundered => write!(fmt, "Floundered"),
        }
    }
}

struct ForestSolver<'me, I: Interner, C: Context<I>, CO: ContextOps<I, C>> {
    forest: &'me mut Forest<I, C>,
    context: &'me CO,
    table: TableIndex,
    answer: AnswerIndex,
}

impl<'me, I: Interner, C: Context<I>, CO: ContextOps<I, C>> AnswerStream<I, C> for ForestSolver<'me, I, C, CO> {
    /// # Panics
    ///
    /// Panics if a negative cycle was detected.
    fn peek_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<I> {
        loop {
            match self
                .forest
                .root_answer(self.context, self.table, self.answer)
            {
                Ok(answer) => {
                    return AnswerResult::Answer(answer.clone());
                }

                Err(RootSearchFail::InvalidAnswer) => {
                    self.answer.increment();
                }
                Err(RootSearchFail::Floundered) => {
                    return AnswerResult::Floundered;
                }

                Err(RootSearchFail::NoMoreSolutions) => {
                    return AnswerResult::NoMoreSolutions;
                }

                Err(RootSearchFail::QuantumExceeded) => {
                    if !should_continue() {
                        return AnswerResult::QuantumExceeded;
                    }
                }

                Err(RootSearchFail::NegativeCycle) => {
                    // Negative cycles *ought* to be avoided by construction. Hence panic
                    // if we find one, as that likely indicates a problem in the chalk-solve
                    // lowering rules. (In principle, we could propagate this error out,
                    // and let chalk-solve do the asserting, but that seemed like it would
                    // complicate the function signature more than it's worth.)
                    panic!("negative cycle was detected");
                }
            }
        }
    }

    fn next_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<I> {
        let answer = self.peek_answer(should_continue);
        self.answer.increment();
        answer
    }

    fn any_future_answer(&self, test: impl Fn(&Substitution<I>) -> bool) -> bool {
        self.forest.any_future_answer(self.table, self.answer, test)
    }
}
