use chalk_parse::ast::{self, Span};
use formula::*;

use self::environment::LowerEnvironment;
use self::lower_clause::LowerClause;
use self::lower_goal::LowerGoal;

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
    let mut env = LowerEnvironment::new();
    let clausess: Vec<Vec<_>> = try!(program.items
        .iter()
        .map(|item| item.lower_clause(&mut env))
        .collect());
    Ok(clausess.into_iter()
       .flat_map(|v| v)
       .collect())
}

pub fn lower_goal(fact: &ast::Fact) -> LowerResult<Goal<Application>> {
    let mut env = LowerEnvironment::new();
    fact.lower_goal(&mut env)
}
