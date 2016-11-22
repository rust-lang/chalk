use formula::*;
use solve::Environment;
use std::iter::repeat;
use std::sync::Arc;

use super::Solver;

fn solve(clauses: Vec<Clause<Application>>,
         goal: Goal<Application>,
         expected_solutions: Vec<&str>) {
    let root_environment = Arc::new(Environment::new(None, clauses));
    let solutions = Solver::solve(root_environment, goal);

    let is_match: Vec<bool> = expected_solutions.iter()
        .zip(&solutions)
        .map(|(e, a)| e == a)
        .collect();
    let is_match =
        || is_match.iter().cloned().chain(repeat(false)).map(|b| if b { 'x' } else { ' ' });

    debug!("expected_solutions:");
    for (solution, m) in expected_solutions.iter().zip(is_match()) {
        debug!("- [{}] {}", m, solution);
    }

    debug!("actual_solutions:");
    for (solution, m) in solutions.iter().zip(is_match()) {
        debug!("- [{}] {}", m, solution);
    }

    assert_eq!(expected_solutions, solutions);
}

#[test]
fn simple_fail() {
    solve(vec![], goal!(exists(1) (apply "foo" (bound 0))), vec![]);
}

#[test]
fn forall_in_clause() {
    solve(vec![],
          goal!(exists(1) (implies (forall(1) (apply "foo" (bound 0))) =>
                           (apply "foo" (bound 0)))),
          vec![r#"implies(forall(A -> "foo"(A)) => "foo"(?0))"#]);
}

#[test]
fn one_clause() {
    solve(vec![],
          goal!(exists(1) (implies (apply "foo" (apply "bar")) =>
                           (apply "foo" (bound 0)))),
          vec![r#"implies("foo"("bar") => "foo"("bar"))"#]);
}

#[test]
fn two_clause_in_env() {
    solve(vec![clause!(apply "foo" (apply "bar")),
               clause!(apply "foo" (apply "baz"))],
          goal!(exists(1) (apply "foo" (bound 0))),
          vec![r#""foo"("bar")"#,
               r#""foo"("baz")"#]);
}

#[test]
fn enumerate_ancestors() {
    solve(vec![clause!(apply "parent" (apply "n") (apply "d")),
               clause!(apply "parent" (apply "c") (apply "n")),
               // ancestor(A, B) :- parent(A, B).
               clause!(forall(2) (implies (apply "parent" (bound 0) (bound 1)) =>
                                  (apply "ancestor" (bound 0) (bound 1)))),
               // ancestor(A, C) :- parent(A, B), ancestor(B, C).
               clause!(forall(3) (implies
                                  (and
                                   (apply "parent" (bound 0) (bound 1))
                                   (apply "ancestor" (bound 1) (bound 2))) =>
                                  (apply "ancestor" (bound 0) (bound 2)))),
               ],
          goal!(exists(1) (apply "ancestor" (bound 0) (apply "d"))),
          vec![r#""ancestor"("n", "d")"#,
               r#""ancestor"("c", "d")"#]);
}

#[test]
fn forall_fails() {
    solve(vec![clause!(apply "foo" (apply "bar")),
               clause!(apply "foo" (apply "baz"))],
          goal!(forall(1) (apply "foo" (bound 0))),
          vec![]);
}

#[test]
fn for_all_clause_for_all_goal() {
    solve(vec![clause!(forall(1) (apply "foo" (bound 0)))],
          goal!(forall(1) (apply "foo" (bound 0))),
          vec![r#"forall(A -> "foo"(A))"#]);
}
