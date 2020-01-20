use crate::context::Context;
use crate::strand::CanonicalStrand;
use crate::Answer;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use std::collections::VecDeque;
use std::mem;

pub(crate) struct Table<C: Context> {
    /// The goal this table is trying to solve (also the key to look
    /// it up).
    pub(crate) table_goal: C::UCanonicalGoalInEnvironment,

    /// A goal is coinductive if it can assume itself to be true, more
    /// or less. This is true for auto traits.
    pub(crate) coinductive_goal: bool,

    /// True if this table is floundered, meaning that it doesn't have
    /// enough types specified for us to solve.
    floundered: bool,

    /// Stores the answers that we have found thus far. When we get a request
    /// for an answer N, we will first check this vector.
    answers: Vec<Answer<C>>,

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
    answers_hash: FxHashMap<C::CanonicalAnswerSubst, bool>,

    /// Stores the active strands that we can "pull on" to find more
    /// answers.
    strands: VecDeque<CanonicalStrand<C>>,
}

index_struct! {
    pub(crate) struct AnswerIndex {
        value: usize,
    }
}

impl<C: Context> Table<C> {
    pub(crate) fn new(
        table_goal: C::UCanonicalGoalInEnvironment,
        coinductive_goal: bool,
    ) -> Table<C> {
        Table {
            table_goal,
            coinductive_goal,
            answers: Vec::new(),
            floundered: false,
            answers_hash: FxHashMap::default(),
            strands: VecDeque::new(),
        }
    }

    /// Push a strand to the back of the queue of strands to be processed.
    pub(crate) fn enqueue_strand(&mut self, strand: CanonicalStrand<C>) {
        self.strands.push_back(strand);
    }

    pub(crate) fn strands_mut(&mut self) -> impl Iterator<Item = &mut CanonicalStrand<C>> {
        self.strands.iter_mut()
    }

    pub(crate) fn strands(&self) -> impl Iterator<Item = &CanonicalStrand<C>> {
        self.strands.iter()
    }

    pub(crate) fn take_strands(&mut self) -> VecDeque<CanonicalStrand<C>> {
        mem::replace(&mut self.strands, VecDeque::new())
    }

    /// Remove the next strand from the queue as long as it meets the
    /// given criteria.
    pub(crate) fn dequeue_next_strand_if(
        &mut self,
        test: impl Fn(&CanonicalStrand<C>) -> bool,
    ) -> Option<CanonicalStrand<C>> {
        let strand = self.strands.pop_front();
        if let Some(strand) = strand {
            if test(&strand) {
                return Some(strand);
            }
            self.strands.push_front(strand);
        }
        None
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
    pub(super) fn push_answer(&mut self, answer: Answer<C>) -> Option<AnswerIndex> {
        assert!(!self.floundered);

        debug_heading!("push_answer(answer={:?})", answer);
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
            "new answer to table with goal {:?}: answer={:?}",
            self.table_goal, answer,
        );
        if !added {
            return None;
        }

        let index = self.answers.len();
        self.answers.push(answer);
        Some(AnswerIndex::from(index))
    }

    pub(super) fn answer(&self, index: AnswerIndex) -> Option<&Answer<C>> {
        self.answers.get(index.value)
    }

    pub(super) fn next_answer_index(&self) -> AnswerIndex {
        AnswerIndex::from(self.answers.len())
    }
}

impl AnswerIndex {
    pub(crate) const ZERO: AnswerIndex = AnswerIndex { value: 0 };
}
