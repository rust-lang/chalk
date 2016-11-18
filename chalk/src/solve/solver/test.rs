use formula::*;
use solve::Environment;
use std::iter::repeat;
use std::sync::Arc;

use super::Solver;

fn root_environment() -> Arc<Environment> {
    Arc::new(Environment::new(None, vec![]))
}

fn solve(goal: Goal<Application>,
         expected_solutions: Vec<&str>)
{
    let solutions = Solver::solve(root_environment(), goal);

    let is_match: Vec<bool> =
        expected_solutions.iter()
                          .zip(&solutions)
                          .map(|(e, a)| e == a)
                          .collect();
    let is_match = || {
        is_match.iter().cloned().chain(repeat(false)).map(|b| if b {
            'x'
        } else {
            ' '
        })
    };

    println!("expected_solutions:");
    for (solution, m) in expected_solutions.iter().zip(is_match()) {
        println!("- [{}] {}", m, solution);
    }

    println!("actual_solutions:");
    for (solution, m) in solutions.iter().zip(is_match()) {
        println!("- [{}] {}", m, solution);
    }

    assert_eq!(expected_solutions, solutions);
}

#[test]
fn simple_fail() {
    solve(goal!(exists(1) (apply "foo" (bound 0))), vec![]);
}

#[test]
fn forall_in_clause() {
    solve(goal!(exists(1) (implies (forall(1) (apply "foo" (bound 0))) =>
                           (apply "foo" (bound 0)))),
          vec![r#"implies(forall(A -> "foo"(A)) => "foo"(?0))"#]);
}

#[test]
fn one_clause() {
    solve(goal!(exists(1) (implies (apply "foo" (apply "bar")) =>
                           (apply "foo" (bound 0)))),
          vec![r#"implies("foo"("bar") => "foo"("bar"))"#]);
}
