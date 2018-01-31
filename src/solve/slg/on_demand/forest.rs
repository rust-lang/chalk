use ir::*;
use solve::Solution;
use solve::slg::aggregate;
use solve::slg::{DepthFirstNumber, SimplifiedAnswer, TableIndex, UCanonicalGoal};
use solve::slg::on_demand::logic::SearchFail;
use solve::slg::on_demand::stack::Stack;
use solve::slg::on_demand::tables::Tables;
use solve::slg::on_demand::table::{Answer, AnswerIndex};
use std::sync::Arc;

pub struct Forest {
    crate program: Arc<ProgramEnvironment>,
    crate tables: Tables,
    crate stack: Stack,
    crate max_size: usize,

    dfn: DepthFirstNumber,
}

impl Forest {
    crate fn new(program: &Arc<ProgramEnvironment>, max_size: usize) -> Self {
        Forest {
            program: program.clone(),
            tables: Tables::default(),
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
    pub(super) fn force_answers(
        &mut self,
        goal: UCanonicalGoal,
        num_answers: usize,
    ) -> Vec<Answer> {
        let table = self.get_or_create_table_for_ucanonical_goal(goal);
        let mut answers = Vec::with_capacity(num_answers);
        for i in 0..num_answers {
            let i = AnswerIndex::from(i);
            loop {
                match self.ensure_answer(table, i) {
                    Ok(()) => break,
                    Err(SearchFail::QuantumExceeded) => continue,
                    Err(SearchFail::NoMoreSolutions) => return answers,
                    Err(SearchFail::Cycle(_)) => panic!("unresolved cycle"),
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
    pub fn iter_answers<'f>(
        &'f mut self,
        goal: &UCanonicalGoal,
    ) -> impl Iterator<Item = SimplifiedAnswer> + 'f {
        let table = self.get_or_create_table_for_ucanonical_goal(goal.clone());
        let answer = AnswerIndex::ZERO;
        ForestSolver { forest: self, table, answer }
    }

    /// Solves a given goal, producing the solution. This will do only
    /// as much work towards `goal` as it has to (and that works is
    /// cached for future attempts).
    pub fn solve(
        &mut self,
        goal: &UCanonicalGoal,
    ) -> Option<Solution> {
        aggregate::make_solution(&goal.canonical, self.iter_answers(goal))
    }
}

struct ForestSolver<'forest> {
    forest: &'forest mut Forest,
    table: TableIndex,
    answer: AnswerIndex,
}

impl<'forest> Iterator for ForestSolver<'forest> {
    type Item = SimplifiedAnswer;

    fn next(&mut self) -> Option<SimplifiedAnswer> {
        loop {
            match self.forest.ensure_answer(self.table, self.answer) {
                Ok(()) => {
                    let answer = self.forest.answer(self.table, self.answer);

                    // FIXME -- if answer has delayed literals, we
                    // *should* try to simplify here (which might
                    // involve forcing `table` and its dependencies to
                    // completion. But instead we'll err on the side
                    // of ambiguity for now. This will sometimes lose
                    // us completeness around negative reasoning
                    // (we'll give ambig when we could have given a
                    // concrete yes/no answer).

                    let simplified_answer = SimplifiedAnswer {
                        subst: answer.subst.clone(),
                        ambiguous: !answer.delayed_literals.is_empty(),
                    };

                    self.answer.increment();

                    return Some(simplified_answer);
                }

                Err(SearchFail::NoMoreSolutions) => {
                    return None;
                }

                Err(SearchFail::QuantumExceeded) => {
                }

                Err(SearchFail::Cycle(..)) => {
                    panic!("cycle should not happen for top-most goal")
                }
            }
        }
    }
}


