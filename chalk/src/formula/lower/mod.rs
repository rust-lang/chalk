use chalk_parse::ast::{self, Span};
use super::goal::Goal;
use super::clause::Clause;

pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

pub enum ErrorKind {
    UnknownVariable(ast::Variable),
}

pub type LowerResult<L> = Result<L, Error>;

pub trait LowerClause<L> {
    fn lower_clause(&self) -> LowerResult<Clause<L>>;
}

pub trait LowerGoal<L> {
    fn lower_clause(&self) -> LowerResult<Goal<L>>;
}

mod environment;
mod lower_leaf;
