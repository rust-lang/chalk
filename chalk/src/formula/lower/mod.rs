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
    use std::collections::HashSet;

    let mut env = LowerEnvironment::new(path.to_string());

    // Bring all free variables into scope. We will wrap them in
    // existentials.  (i.e., foo(X) is short for exists(X -> foo(X))).
    let mut count = 0;
    let mut set = HashSet::new();
    fact.for_each_free_variable(&mut |_span, v| {
        if set.insert(v.id) {
            count += 1;
            env.push_bound_name(v);
        }
    });

    let goal = fact.lower_goal(&mut env)?;
    Ok(goal.in_exists(count))
}
