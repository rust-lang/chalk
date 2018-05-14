use crate::{DepthFirstNumber, SimplifiedAnswer, TableIndex};
use crate::context::prelude::*;
use crate::context::AnswerStream;
use crate::logic::RootSearchFail;
use crate::stack::{Stack, StackIndex};
use crate::tables::Tables;
use crate::table::{Answer, AnswerIndex};

pub struct Forest<C: Context> {
    #[allow(dead_code)]
    crate context: C,
    crate tables: Tables<C>,
    crate stack: Stack,

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

    // Gets the next depth-first number. This number never decreases.
    pub(super) fn next_dfn(&mut self) -> DepthFirstNumber {
        self.dfn.next()
    }

    /// Finds the first N answers, looping as much as needed to get
    /// them.
    ///
    /// Thanks to subgoal abstraction and so forth, this should always
    /// terminate.
    pub fn force_answers(
        &mut self,
        goal: C::UCanonicalGoalInEnvironment,
        num_answers: usize,
    ) -> Vec<Answer<C>> {
        let table = self.get_or_create_table_for_ucanonical_goal(goal);
        let mut answers = Vec::with_capacity(num_answers);
        for i in 0..num_answers {
            let i = AnswerIndex::from(i);
            loop {
                match self.ensure_root_answer(table, i) {
                    Ok(()) => break,
                    Err(RootSearchFail::QuantumExceeded) => continue,
                    Err(RootSearchFail::NoMoreSolutions) => return answers,
                }
            }

            answers.push(self.answer(table, i).clone());
        }

        answers
    }

    /// Returns a "solver" for a given goal in the form of an
    /// iterator. Each time you invoke `next`, it will do the work to
    /// extract one more answer. These answers are cached in between
    /// invocations. Invoking `next` fewer times is preferable =)
    fn iter_answers<'f>(
        &'f mut self,
        goal: &C::UCanonicalGoalInEnvironment,
    ) -> impl AnswerStream<C> + 'f {
        let table = self.get_or_create_table_for_ucanonical_goal(goal.clone());
        let answer = AnswerIndex::ZERO;
        ForestSolver {
            forest: self,
            table,
            answer,
        }
    }

    /// Solves a given goal, producing the solution. This will do only
    /// as much work towards `goal` as it has to (and that works is
    /// cached for future attempts).
    pub fn solve(&mut self, goal: &C::UCanonicalGoalInEnvironment) -> Option<C::Solution> {
        self.context.clone().make_solution(C::canonical(&goal), self.iter_answers(goal))
    }

    /// True if all the tables on the stack starting from `depth` and
    /// continuing until the top of the stack are coinductive.
    ///
    /// Example: Given a program like:
    ///
    /// ```
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
        self.stack.top_of_stack_from(depth).all(|d| {
            let table = self.stack[d].table;
            self.tables[table].coinductive_goal
        })
    }

    /// Useful for testing.
    pub fn num_cached_answers_for_goal(&mut self, goal: &C::UCanonicalGoalInEnvironment) -> usize {
        let table = self.get_or_create_table_for_ucanonical_goal(goal.clone());
        self.tables[table].num_cached_answers()
    }
}

struct ForestSolver<'forest, C: Context + 'forest> {
    forest: &'forest mut Forest<C>,
    table: TableIndex,
    answer: AnswerIndex,
}

impl<'forest, C> AnswerStream<C> for ForestSolver<'forest, C>
where
    C: Context,
{
    fn peek_answer(&mut self) -> Option<SimplifiedAnswer<C>> {
        loop {
            match self.forest.ensure_root_answer(self.table, self.answer) {
                Ok(()) => {
                    let answer = self.forest.answer(self.table, self.answer);

                    // FIXME(rust-lang-nursery/chalk#79) -- if answer
                    // has delayed literals, we *should* try to
                    // simplify here (which might involve forcing
                    // `table` and its dependencies to completion. But
                    // instead we'll err on the side of ambiguity for
                    // now. This will sometimes lose us completeness
                    // around negative reasoning (we'll give ambig
                    // when we could have given a concrete yes/no
                    // answer).

                    let simplified_answer = SimplifiedAnswer {
                        subst: answer.subst.clone(),
                        ambiguous: !answer.delayed_literals.is_empty(),
                    };

                    return Some(simplified_answer);
                }

                Err(RootSearchFail::NoMoreSolutions) => {
                    return None;
                }

                Err(RootSearchFail::QuantumExceeded) => {}
            }
        }
    }

    fn next_answer(&mut self) -> Option<SimplifiedAnswer<C>> {
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
