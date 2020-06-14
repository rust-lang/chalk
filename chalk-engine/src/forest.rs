use crate::context::{AnswerResult, AnswerStream, Context, ContextOps};
use crate::logic::RootSearchFail;
use crate::table::AnswerIndex;
use crate::tables::Tables;
use crate::{TableIndex, TimeStamp};

use chalk_ir::interner::Interner;
use chalk_ir::{Goal, InEnvironment, Substitution, UCanonical};
use tracing::debug;

pub struct Forest<I: Interner, C: Context<I>> {
    pub(crate) tables: Tables<I>,

    /// This is a clock which always increases. It is
    /// incremented every time a new subgoal is followed.
    /// This effectively gives us way to track what depth
    /// and loop a table or strand was last followed.
    pub(crate) clock: TimeStamp,
    _context: std::marker::PhantomData<C>,
}

impl<I: Interner, C: Context<I>> Forest<I, C> {
    pub fn new() -> Self {
        Forest {
            tables: Tables::new(),
            clock: TimeStamp::default(),
            _context: std::marker::PhantomData,
        }
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
    pub fn iter_answers<'f>(
        &'f mut self,
        context: &'f impl ContextOps<I, C>,
        goal: &UCanonical<InEnvironment<Goal<I>>>,
    ) -> impl AnswerStream<I> + 'f {
        let table = self.get_or_create_table_for_ucanonical_goal(context, goal.clone());
        let answer = AnswerIndex::ZERO;
        ForestSolver {
            forest: self,
            context,
            table,
            answer,
            _context: std::marker::PhantomData::<C>,
        }
    }
}

struct ForestSolver<'me, I: Interner, C: Context<I>, CO: ContextOps<I, C>> {
    forest: &'me mut Forest<I, C>,
    context: &'me CO,
    table: TableIndex,
    answer: AnswerIndex,
    _context: std::marker::PhantomData<C>,
}

impl<'me, I: Interner, C: Context<I>, CO: ContextOps<I, C>> AnswerStream<I>
    for ForestSolver<'me, I, C, CO>
{
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
                    debug!("Answer: {:?}", &answer);
                    return AnswerResult::Answer(answer);
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
