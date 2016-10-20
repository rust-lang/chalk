use chalk_parse::ast::{self, Span};

pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

pub enum ErrorKind {
    UnknownVariable(ast::Variable),
    OrInClause,
    ExistsInClause,
}

pub type LowerResult<L> = Result<L, Error>;

mod environment;
mod lower_leaf;
mod lower_clause;
mod lower_goal;
