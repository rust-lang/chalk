use chalk_parse::ast::{self, Span};
use formula::clause::Clause;
use formula::leaf::Leaf;

use self::environment::Environment;
use self::lower_clause::LowerClause;

#[derive(Clone, Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Clone, Debug)]
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
#[cfg(test)]
mod test;

pub fn lower_program(program: &ast::Program) -> LowerResult<Vec<Clause<Leaf>>> {
    let mut env = Environment::new();
    program.items
        .iter()
        .map(|item| item.lower_clause(&mut env))
        .collect()
}
