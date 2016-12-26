use chalk_parse::ast::{self, Span};
use std::error;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Error {
    pub path: String,
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UnknownVariable(ast::Variable),
    IllegalClause,
    NoOperator,
}

pub type LowerResult<L> = Result<L, Error>;

impl error::Error for Error {
    fn description(&self) -> &str {
        "lower error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // FIXME-- convert byte offsets into line/col, better serialization of `ErrorKind`
        write!(fmt, "error at {}:({}..{}): {:?}",
               self.path,
               self.span.lo,
               self.span.hi,
               self.kind)
    }
}
