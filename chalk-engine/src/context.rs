//! Defines traits used to embed the chalk-engine in another crate.
//!
//! chalk and rustc both define types which implement the traits in this
//! module. This allows each user of chalk-engine to define their own
//! `DomainGoal` type, add arena lifetime parameters, and more. See
//! [`Context`] trait for a list of types.

use crate::CompleteAnswer;
use chalk_ir::interner::Interner;
use chalk_ir::Substitution;
use std::fmt::Debug;

pub enum AnswerResult<I: Interner> {
    /// The next available answer.
    Answer(CompleteAnswer<I>),

    /// No answer could be returned because there are no more solutions.
    NoMoreSolutions,

    /// No answer could be returned because the goal has floundered.
    Floundered,

    // No answer could be returned *yet*, because we exceeded our
    // quantum (`should_continue` returned false).
    QuantumExceeded,
}

impl<I: Interner> AnswerResult<I> {
    pub fn is_answer(&self) -> bool {
        matches!(self, Self::Answer(_))
    }

    pub fn answer(self) -> CompleteAnswer<I> {
        match self {
            Self::Answer(answer) => answer,
            _ => panic!("Not an answer."),
        }
    }

    pub fn is_no_more_solutions(&self) -> bool {
        matches!(self, Self::NoMoreSolutions)
    }

    pub fn is_quantum_exceeded(&self) -> bool {
        matches!(self, Self::QuantumExceeded)
    }
}

impl<I: Interner> Debug for AnswerResult<I> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnswerResult::Answer(answer) => write!(fmt, "{:?}", answer),
            AnswerResult::Floundered => write!(fmt, "Floundered"),
            AnswerResult::NoMoreSolutions => write!(fmt, "None"),
            AnswerResult::QuantumExceeded => write!(fmt, "QuantumExceeded"),
        }
    }
}

pub trait AnswerStream<I: Interner> {
    /// Gets the next answer for a given goal, but doesn't increment the answer index.
    /// Calling this or `next_answer` again will give the same answer.
    fn peek_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<I>;

    /// Gets the next answer for a given goal, incrementing the answer index.
    /// Calling this or `peek_answer` again will give the next answer.
    fn next_answer(&mut self, should_continue: impl Fn() -> bool) -> AnswerResult<I>;

    /// Invokes `test` with each possible future answer, returning true immediately
    /// if we find any answer for which `test` returns true.
    fn any_future_answer(&self, test: impl Fn(&Substitution<I>) -> bool) -> bool;
}
