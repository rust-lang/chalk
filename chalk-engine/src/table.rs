use crate::{DelayedLiteralSet, DelayedLiteralSets};
use crate::context::prelude::*;
use crate::strand::CanonicalStrand;
use fxhash::FxHashMap;
use std::collections::VecDeque;
use std::collections::hash_map::Entry;
use std::mem;
use std::iter;

crate struct Table<C: Context> {
    /// The goal this table is trying to solve (also the key to look
    /// it up).
    crate table_goal: C::UCanonicalGoalInEnvironment,

    /// A goal is coinductive if it can assume itself to be true, more
    /// or less. This is true for auto traits.
    crate coinductive_goal: bool,

    /// Stores the answers that we have found thus far. When we get a request
    /// for an answer N, we will first check this vector.
    answers: Vec<Answer<C>>,

    /// An alternative storage for the answers we have so far, used to
    /// detect duplicates. Not every answer in `answers` will be
    /// represented here -- we discard answers from `answers_hash`
    /// (but not `answers`) when better answers arrive (in particular,
    /// answers with fewer delayed literals).
    answers_hash: FxHashMap<C::CanonicalConstrainedSubst, DelayedLiteralSets<C>>,

    /// Stores the active strands that we can "pull on" to find more
    /// answers.
    strands: VecDeque<CanonicalStrand<C>>,
}

index_struct! {
    crate struct AnswerIndex {
        value: usize,
    }
}


/// An "answer" in the on-demand solver corresponds to a fully solved
/// goal for a particular table (modulo delayed literals). It contains
/// a substitution
#[derive(Clone, Debug)]
pub struct Answer<C: Context> {
    crate subst: C::CanonicalConstrainedSubst,
    crate delayed_literals: DelayedLiteralSet<C>,
}

impl<C: Context> Table<C> {
    crate fn new(table_goal: C::UCanonicalGoalInEnvironment, coinductive_goal: bool) -> Table<C> {
        Table {
            table_goal,
            coinductive_goal,
            answers: Vec::new(),
            answers_hash: FxHashMap::default(),
            strands: VecDeque::new(),
        }
    }

    crate fn push_strand(&mut self, strand: CanonicalStrand<C>) {
        self.strands.push_back(strand);
    }

    crate fn extend_strands(&mut self, strands: impl IntoIterator<Item = CanonicalStrand<C>>) {
        self.strands.extend(strands);
    }

    crate fn strands_mut(&mut self) -> impl Iterator<Item = &mut CanonicalStrand<C>> {
        self.strands.iter_mut()
    }

    crate fn take_strands(&mut self) -> VecDeque<CanonicalStrand<C>> {
        mem::replace(&mut self.strands, VecDeque::new())
    }

    crate fn pop_next_strand(&mut self) -> Option<CanonicalStrand<C>> {
        self.strands.pop_front()
    }

    /// Adds `answer` to our list of answers, unless it (or some
    /// better answer) is already present. An answer A is better than
    /// an answer B if their substitutions are the same, but A has a subset
    /// of the delayed literals that B does.
    ///
    /// Returns true if `answer` was added.
    pub(super) fn push_answer(&mut self, answer: Answer<C>) -> bool {
        debug_heading!("push_answer(answer={:?})", answer);
        debug!(
            "pre-existing entry: {:?}",
            self.answers_hash.get(&answer.subst)
        );

        if answer.delayed_literals.is_empty() {
            match self.answers_hash.entry(answer.subst.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(DelayedLiteralSets::None);
                }

                Entry::Occupied(mut entry) => {
                    if entry.get().is_empty() {
                        return false;
                    }

                    entry.insert(DelayedLiteralSets::None);
                }
            }

            info!(
                "new answer to table with goal {:?}: answer={:?}",
                self.table_goal, answer,
            );
            self.answers.push(answer);
            return true;
        }

        match self.answers_hash.entry(answer.subst.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(DelayedLiteralSets::Some(
                    iter::once(answer.delayed_literals.clone()).collect(),
                ));
            }

            Entry::Occupied(mut entry) => {
                match entry.get_mut() {
                    DelayedLiteralSets::None => {
                        return false;
                    }

                    DelayedLiteralSets::Some(sets) => {
                        // look for an older answer that is better than this one
                        if sets.iter()
                            .any(|set| set.is_subset(&answer.delayed_literals))
                        {
                            return false;
                        }

                        // discard older answers where this new answer is better
                        sets.retain(|set| !answer.delayed_literals.is_subset(set));

                        sets.insert(answer.delayed_literals.clone());
                    }
                }
            }
        }

        info!(
            "new answer to table with goal {:?}: answer={:?}",
            self.table_goal, answer,
        );
        self.answers.push(answer);
        true
    }

    pub(super) fn answer(&self, index: AnswerIndex) -> Option<&Answer<C>> {
        self.answers.get(index.value)
    }

    /// Useful for testing.
    pub fn num_cached_answers(&self) -> usize {
        self.answers.len()
    }

    pub(super) fn next_answer_index(&self) -> AnswerIndex {
        AnswerIndex::from(self.answers.len())
    }
}

impl AnswerIndex {
    crate const ZERO: AnswerIndex = AnswerIndex { value: 0 };
}

impl<C: Context> Answer<C> {
    /// An "unconditional" answer is one that must be true -- this is
    /// the case so long as we have no delayed literals.
    pub(super) fn is_unconditional(&self) -> bool {
        self.delayed_literals.is_empty()
    }
}
