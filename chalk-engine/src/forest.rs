use crate::context::prelude::*;
use crate::context::AnswerStream;
use crate::logic::RootSearchFail;
use crate::stack::{Stack, StackIndex};
use crate::table::AnswerIndex;
use crate::tables::Tables;
use crate::Answer;
use crate::{DepthFirstNumber, TableIndex};

pub struct Forest<C: Context> {
    context: C,
    pub(crate) tables: Tables<C>,
    pub(crate) stack: Stack,

    dfn: DepthFirstNumber,
}

impl<C: Context> Forest<C> {
    pub fn new(context: C) -> Self {
        Forest {
            context,
            tables: Tables::new(),
            stack: Stack::default(),
            dfn: DepthFirstNumber::MIN,
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

    // Gets the next depth-first number. This number never decreases.
    pub(super) fn next_dfn(&mut self) -> DepthFirstNumber {
        self.dfn.next()
    }

    /// Finds the first N answers, looping as much as needed to get
    /// them. Returns `None` if the result flounders.
    ///
    /// Thanks to subgoal abstraction and so forth, this should always
    /// terminate.
    ///
    /// # Panics
    ///
    /// Panics if a negative cycle was detected.
    pub fn force_answers(
        &mut self,
        context: &impl ContextOps<C>,
        goal: C::UCanonicalGoalInEnvironment,
        num_answers: usize,
    ) -> Option<Vec<Answer<C>>> {
        let table = self.get_or_create_table_for_ucanonical_goal(context, goal);
        let mut answers = Vec::with_capacity(num_answers);
        for i in 0..num_answers {
            let i = AnswerIndex::from(i);
            loop {
                match self.ensure_root_answer(context, table, i) {
                    Ok(()) => break,
                    Err(RootSearchFail::Floundered) => return None,
                    Err(RootSearchFail::QuantumExceeded) => continue,
                    Err(RootSearchFail::NoMoreSolutions) => return Some(answers),
                    Err(RootSearchFail::NegativeCycle) => {
                        panic!("negative cycle was detected");
                    }
                }
            }

            answers.push(self.answer(table, i).clone());
        }

        Some(answers)
    }

    /// Returns a "solver" for a given goal in the form of an
    /// iterator. Each time you invoke `next`, it will do the work to
    /// extract one more answer. These answers are cached in between
    /// invocations. Invoking `next` fewer times is preferable =)
    fn iter_answers<'f>(
        &'f mut self,
        context: &'f impl ContextOps<C>,
        goal: &C::UCanonicalGoalInEnvironment,
    ) -> impl AnswerStream<C> + 'f {
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
        context: &impl ContextOps<C>,
        goal: &C::UCanonicalGoalInEnvironment,
    ) -> Option<C::Solution> {
        context.make_solution(C::canonical(&goal), self.iter_answers(context, goal))
    }

    /// Solves a given goal, producing the solution. This will do only
    /// as much work towards `goal` as it has to (and that works is
    /// cached for future attempts). Calls provided function `f` to
    /// iterate over multiple solutions until the function return `false`.
    pub fn solve_multiple(
        &mut self,
        context: &impl ContextOps<C>,
        goal: &C::UCanonicalGoalInEnvironment,
        mut f: impl FnMut(C::CanonicalConstrainedSubst, bool) -> bool,
    ) -> bool {
        let mut answers = self.iter_answers(context, goal);
        while let Some(answer) = answers.next_answer() {
            if !f(
                context.constrained_subst_from_answer(answer),
                answers.peek_answer().is_some(),
            ) {
                return false;
            }
        }
        return true;
    }

    /// True if all the tables on the stack starting from `depth` and
    /// continuing until the top of the stack are coinductive.
    ///
    /// Example: Given a program like:
    ///
    /// ```notrust
    /// struct Foo { a: Option<Box<Bar>> }
    /// struct Bar { a: Option<Box<Foo>> }
    /// trait XXX { }
    /// impl<T: Send> XXX for T { }
    /// ```
    ///
    /// and then a goal of `Foo: XXX`, we would eventually wind up
    /// with a stack like this:
    ///
    /// | StackIndex | Table Goal  |
    /// | ---------- | ----------- |
    /// | 0          | `Foo: XXX`  |
    /// | 1          | `Foo: Send` |
    /// | 2          | `Bar: Send` |
    ///
    /// Here, the top of the stack is `Bar: Send`. And now we are
    /// asking `top_of_stack_is_coinductive_from(1)` -- the answer
    /// would be true, since `Send` is an auto trait, which yields a
    /// coinductive goal. But `top_of_stack_is_coinductive_from(0)` is
    /// false, since `XXX` is not an auto trait.
    pub(super) fn top_of_stack_is_coinductive_from(&self, depth: StackIndex) -> bool {
        StackIndex::iterate_range(self.stack.top_of_stack_from(depth)).all(|d| {
            let table = self.stack[d].table;
            self.tables[table].coinductive_goal
        })
    }

    /// Useful for testing.
    pub fn num_cached_answers_for_goal(
        &mut self,
        context: &impl ContextOps<C>,
        goal: &C::UCanonicalGoalInEnvironment,
    ) -> usize {
        let table = self.get_or_create_table_for_ucanonical_goal(context, goal.clone());
        self.tables[table].num_cached_answers()
    }
}

struct ForestSolver<'me, C: Context, CO: ContextOps<C>> {
    forest: &'me mut Forest<C>,
    context: &'me CO,
    table: TableIndex,
    answer: AnswerIndex,
}

impl<'me, C: Context, CO: ContextOps<C>> AnswerStream<C> for ForestSolver<'me, C, CO> {
    /// # Panics
    ///
    /// Panics if a negative cycle was detected.
    fn peek_answer(&mut self) -> Option<Answer<C>> {
        loop {
            match self
                .forest
                .ensure_root_answer(self.context, self.table, self.answer)
            {
                Ok(()) => {
                    let answer = self.forest.answer(self.table, self.answer);
                    return Some(answer.clone());
                }

                Err(RootSearchFail::Floundered) => {
                    let table_goal = &self.forest.tables[self.table].table_goal;
                    return Some(Answer {
                        subst: self.context.identity_constrained_subst(table_goal),
                        ambiguous: true,
                    });
                }

                Err(RootSearchFail::NoMoreSolutions) => {
                    return None;
                }

                Err(RootSearchFail::QuantumExceeded) => {}

                Err(RootSearchFail::NegativeCycle) => {
                    panic!("negative cycle was detected");
                }
            }
        }
    }

    fn next_answer(&mut self) -> Option<Answer<C>> {
        self.peek_answer().map(|answer| {
            self.answer.increment();
            answer
        })
    }

    fn any_future_answer(
        &mut self,
        test: impl FnMut(&C::InferenceNormalizedSubst) -> bool,
    ) -> bool {
        self.forest.any_future_answer(self.table, self.answer, test)
    }
}
