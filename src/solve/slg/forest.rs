use ir::*;
use solve::Solution;
use solve::slg::aggregate;
use solve::slg::context::prelude::*;
use solve::slg::{DepthFirstNumber, SimplifiedAnswer, TableIndex, UCanonicalGoal};
use solve::slg::logic::RootSearchFail;
use solve::slg::stack::{Stack, StackIndex};
use solve::slg::tables::Tables;
use solve::slg::table::AnswerIndex;
use std::sync::Arc;

#[cfg(test)]
use solve::slg::table::Answer;

crate struct Forest<C: Context> {
    #[allow(dead_code)]
    crate context: C,
    crate program: Arc<ProgramEnvironment<DomainGoal>>,
    crate tables: Tables<C>,
    crate stack: Stack,
    crate max_size: usize,

    dfn: DepthFirstNumber,
}

impl<C: Context> Forest<C> {
    /// Convenience fn for solving a root goal. It would be better to
    /// createa a `Forest` so as to enable cahcing between goals, however.
    crate fn solve_root_goal(
        context: C,
        max_size: usize,
        program: &Arc<ProgramEnvironment<DomainGoal>>,
        root_goal: &UCanonicalGoal<DomainGoal>,
    ) -> Option<Solution> {
        let mut forest = Forest::new(context, program, max_size);
        forest.solve(root_goal)
    }

    crate fn new(context: C, program: &Arc<ProgramEnvironment<DomainGoal>>, max_size: usize) -> Self {
        Forest {
            context,
            program: program.clone(),
            tables: Tables::new(),
            stack: Stack::default(),
            max_size,
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
    #[cfg(test)]
    pub(super) fn force_answers(
        &mut self,
        goal: UCanonicalGoal<DomainGoal>,
        num_answers: usize,
    ) -> Vec<Answer> {
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
    pub(super) fn iter_answers<'f>(
        &'f mut self,
        goal: &UCanonicalGoal<DomainGoal>,
    ) -> impl Iterator<Item = SimplifiedAnswer> + 'f {
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
    crate fn solve(&mut self, goal: &UCanonicalGoal<DomainGoal>) -> Option<Solution> {
        aggregate::make_solution(&goal.canonical, self.iter_answers(goal))
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
}

struct ForestSolver<'forest, C: Context + 'forest> {
    forest: &'forest mut Forest<C>,
    table: TableIndex,
    answer: AnswerIndex,
}

impl<'forest, C> Iterator for ForestSolver<'forest, C>
where
    C: Context,
{
    type Item = SimplifiedAnswer;

    fn next(&mut self) -> Option<SimplifiedAnswer> {
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

                    self.answer.increment();

                    return Some(simplified_answer);
                }

                Err(RootSearchFail::NoMoreSolutions) => {
                    return None;
                }

                Err(RootSearchFail::QuantumExceeded) => {}
            }
        }
    }
}
