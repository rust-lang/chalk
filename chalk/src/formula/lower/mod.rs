use chalk_parse::ast;
use formula::*;

use self::environment::LowerEnvironment;
use self::lower_clause::LowerClause;
use self::lower_goal::LowerGoal;
use self::error::LowerResult;

mod environment;
mod error;
mod lower_application;
mod lower_leaf;
mod lower_clause;
mod lower_goal;
#[cfg(test)]
mod test;

pub fn lower_program(path: &str, program: &ast::Program) -> LowerResult<Vec<Clause<Application>>> {
    let mut env = LowerEnvironment::new(path.to_string());
    let clausess: Vec<Vec<_>> = try!(program.items
        .iter()
        .map(|item| item.lower_clause(&mut env))
        .collect());
    Ok(clausess.into_iter()
       .flat_map(|v| v)
       .collect())
}

pub fn lower_goal(path: &str, fact: &ast::Fact) -> LowerResult<Goal<Application>> {
    let mut env = LowerEnvironment::new(path.to_string());
    fact.lower_goal(&mut env)
}
