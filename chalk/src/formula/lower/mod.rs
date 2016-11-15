use chalk_parse::ast::{self, Span};
use formula::*;

use self::environment::Environment;
use self::lower_clause::LowerClause;

#[derive(Clone, Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UnknownVariable(ast::Variable),
    OrInClause,
    ExistsInClause,
    NoOperator,
}

pub type LowerResult<L> = Result<L, Error>;

mod environment;
mod lower_application;
mod lower_leaf;
mod lower_clause;
mod lower_goal;
#[cfg(test)]
mod test;

pub fn lower_program(program: &ast::Program) -> LowerResult<Vec<Clause<Application>>> {
    let mut env = Environment::new();
    let clausess: Vec<Vec<_>> = try!(program.items
        .iter()
        .map(|item| item.lower_clause(&mut env))
        .collect());
    Ok(clausess.into_iter()
       .flat_map(|v| v)
       .collect())
}
