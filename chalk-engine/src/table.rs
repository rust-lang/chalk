use crate::index_struct;
use crate::strand::CanonicalStrand;
use crate::{Answer, AnswerMode};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use std::collections::VecDeque;
use std::mem;

use chalk_ir::interner::Interner;
use chalk_ir::{AnswerSubst, Canonical, Goal, InEnvironment, UCanonical};
use tracing::{debug, info, instrument};

#[derive(Debug)]
pub(crate) struct Table<I: Interner> {
    /// The goal this table is trying to solve (also the key to look
    /// it up).
    pub(crate) table_goal: UCanonical<InEnvironment<Goal<I>>>,

    /// A goal is coinductive if it can assume itself to be true, more
    /// or less. This is true for auto traits.
    pub(crate) coinductive_goal: bool,

    /// True if this table is floundered, meaning that it doesn't have
    /// enough types specified for us to solve.
    floundered: bool,

    /// Stores the answers that we have found thus far. When we get a request
    /// for an answer N, we will first check this vector.
    answers: Vec<Answer<I>>,

    /// An alternative storage for the answers we have so far, used to
    /// detect duplicates. Not every answer in `answers` will be
    /// represented here -- we discard answers from `answers_hash`
    /// (but not `answers`) when better answers arrive (in particular,
    /// answers with no ambiguity).
    ///
    /// FIXME -- Ideally we would exclude the region constraints and
    /// delayed subgoals from the hash, but that's a bit tricky to do
    /// with the current canonicalization setup. It should be ok not
    /// to do so though it can result in more answers than we need.
    answers_hash: FxHashMap<Canonical<AnswerSubst<I>>, bool>,

    /// Stores the active strands that we can "pull on" to find more
    /// answers.
    strands: VecDeque<CanonicalStrand<I>>,

    pub(crate) answer_mode: AnswerMode,
}

index_struct! {
    pub(crate) struct AnswerIndex {
        value: usize,
    }
}

impl<I: Interner> Table<I> {
    pub(crate) fn new(
        table_goal: UCanonical<InEnvironment<Goal<I>>>,
        coinductive_goal: bool,
    ) -> Table<I> {
        Table {
            table_goal,
            coinductive_goal,
            answers: Vec::new(),
            floundered: false,
            answers_hash: FxHashMap::default(),
            strands: VecDeque::new(),
            answer_mode: AnswerMode::Complete,
        }
    }

    /// Push a strand to the back of the queue of strands to be processed.
    pub(crate) fn enqueue_strand(&mut self, strand: CanonicalStrand<I>) {
        self.strands.push_back(strand);
    }

    pub(crate) fn strands_mut(&mut self) -> impl Iterator<Item = &mut CanonicalStrand<I>> {
        self.strands.iter_mut()
    }

    pub(crate) fn strands(&self) -> impl Iterator<Item = &CanonicalStrand<I>> {
        self.strands.iter()
    }

    pub(crate) fn take_strands(&mut self) -> VecDeque<CanonicalStrand<I>> {
        mem::take(&mut self.strands)
    }

    /// Remove the next strand from the queue that meets the given criteria
    pub(crate) fn dequeue_next_strand_that(
        &mut self,
        test: impl Fn(&CanonicalStrand<I>) -> bool,
    ) -> Option<CanonicalStrand<I>> {
        let first = self.strands.iter().position(test);
        if let Some(first) = first {
            self.strands.rotate_left(first);
            self.strands.pop_front()
        } else {
            None
        }
    }

    /// Mark the table as floundered -- this also discards all pre-existing answers,
    /// as they are no longer relevant.
    pub(crate) fn mark_floundered(&mut self) {
        self.floundered = true;
        self.strands = Default::default();
        self.answers = Default::default();
    }

    /// Returns true if the table is floundered.
    pub(crate) fn is_floundered(&self) -> bool {
        self.floundered
    }

    /// Adds `answer` to our list of answers, unless it is already present.
    ///
    /// Returns true if `answer` was added.
    ///
    /// # Panics
    /// This will panic if a previous answer with the same substitution
    /// was marked as ambgiuous, but the new answer is not. No current
    /// tests trigger this case, and assumptions upstream assume that when
    /// `true` is returned here, that a *new* answer was added (instead of an)
    /// existing answer replaced.
    #[instrument(level = "debug", skip(self))]
    pub(super) fn push_answer(&mut self, answer: Answer<I>) -> Option<AnswerIndex> {
        assert!(!self.floundered);
        debug!(
            "pre-existing entry: {:?}",
            self.answers_hash.get(&answer.subst)
        );

        let added = match self.answers_hash.entry(answer.subst.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(answer.ambiguous);
                true
            }

            Entry::Occupied(entry) => {
                let was_ambiguous = entry.get();
                if *was_ambiguous && !answer.ambiguous {
                    panic!("New answer was not ambiguous whereas previous answer was.");
                }
                false
            }
        };

        info!(
            goal = ?self.table_goal, ?answer,
            "new answer to table",
        );
        if !added {
            return None;
        }

        let index = self.answers.len();
        self.answers.push(answer);
        Some(AnswerIndex::from(index))
    }

    pub(super) fn answer(&self, index: AnswerIndex) -> Option<&Answer<I>> {
        self.answers.get(index.value)
    }

    pub(super) fn next_answer_index(&self) -> AnswerIndex {
        AnswerIndex::from(self.answers.len())
    }
}

impl AnswerIndex {
    pub(crate) const ZERO: AnswerIndex = AnswerIndex { value: 0 };
}
