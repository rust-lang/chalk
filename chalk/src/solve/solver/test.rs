use formula::*;
use solve::Environment;
use std::iter::repeat;
use std::sync::Arc;

use super::Solver;
use super::Strategy;
use super::Strategy::*;

fn solve(strategies: &[Strategy],
         clauses: Vec<Clause<Application>>,
         goal: Goal<Application>,
         expected_solutions: Vec<&str>) {
    for &strategy in strategies {
        solve_with_strategy(strategy,
                            clauses.clone(),
                            goal.clone(),
                            expected_solutions.clone())
    }
}

fn solve_all(clauses: Vec<Clause<Application>>,
             goal: Goal<Application>,
             expected_solutions: Vec<&str>) {
    solve(&[DepthFirstSearch, Rust, InOut],
          clauses,
          goal,
          expected_solutions);
}

fn solve_dfs(clauses: Vec<Clause<Application>>,
             goal: Goal<Application>,
             expected_solutions: Vec<&str>) {
    solve(&[Strategy::DepthFirstSearch],
          clauses,
          goal,
          expected_solutions)
}

fn solve_rust(clauses: Vec<Clause<Application>>,
              goal: Goal<Application>,
              expected_solutions: Vec<&str>) {
    solve(&[Strategy::Rust], clauses, goal, expected_solutions)
}

fn solve_with_strategy(strategy: Strategy,
                       clauses: Vec<Clause<Application>>,
                       goal: Goal<Application>,
                       expected_solutions: Vec<&str>) {
    let root_environment = Arc::new(Environment::new(None, clauses));
    let solutions: Vec<_> = Solver::new(&root_environment, &goal, strategy).collect();

    let is_match: Vec<bool> = expected_solutions.iter()
        .zip(&solutions)
        .map(|(e, a)| e == a)
        .collect();
    let is_match =
        || is_match.iter().cloned().chain(repeat(false)).map(|b| if b { 'x' } else { ' ' });

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
    solve_all(vec![], goal!(exists(1) (apply "foo" (bound 0))), vec![]);
}

#[test]
fn forall_in_clause() {
    solve(&[DepthFirstSearch, Rust],
          vec![],
          goal!(exists(1) (implies (forall(1) (apply "foo" (bound 0))) =>
                           (apply "foo" (bound 0)))),
          vec![r#"(forall{?A -> "foo"(?A)} => "foo"(?1))"#]);

    solve(&[InOut],
          vec![],
          goal!(exists(1) (implies (forall(1) (apply "foo" (bound 0))) =>
                           (apply "foo" (bound 0)))),
          vec![r#"(forall{?A -> "foo"(?A)} => "foo"(?0))"#])
}

#[test]
fn one_clause() {
    solve(&[Rust, DepthFirstSearch],
          vec![],
          goal!(exists(1) (implies (apply "foo" (apply "bar")) =>
                           (apply "foo" (bound 0)))),
          vec![r#"("foo"("bar") => "foo"("bar"))"#]);

    solve(&[InOut],
          vec![],
          goal!(exists(1) (implies (apply "foo" (apply "bar")) =>
                           (apply "foo" (bound 0)))),
          vec![r#"<<ambiguous>>"#]);
}

#[test]
fn two_clause_in_env() {
    solve_dfs(vec![clause!(apply "foo" (apply "bar")),
               clause!(apply "foo" (apply "baz"))],
              goal!(exists(1) (apply "foo" (bound 0))),
              vec![r#""foo"("bar")"#,
               r#""foo"("baz")"#]);

    solve_rust(vec![clause!(apply "foo" (apply "bar")),
               clause!(apply "foo" (apply "baz"))],
               goal!(exists(1) (apply "foo" (bound 0))),
               vec![r#"<<ambiguous>>"#]);

    solve_rust(vec![clause!(apply "foo" (apply "bar")),
               clause!(apply "foo" (apply "baz"))],
               goal!((apply "foo" (apply "bar"))),
               vec![r#""foo"("bar")"#]);
}

#[test]
fn enumerate_ancestors() {
    solve_dfs(vec![clause!(apply "parent" (apply "n") (apply "d")),
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
fn enumerate_ancestors_rust() {
    solve_rust(vec![clause!(apply "parent" (apply "n") (apply "d")),
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
               vec![r#"<<ambiguous>>"#]);
}

#[test]
fn forall_fails() {
    solve_all(vec![clause!(apply "foo" (apply "bar")),
                   clause!(apply "foo" (apply "baz"))],
              goal!(forall(1) (apply "foo" (bound 0))),
              vec![]);
}

#[test]
fn for_all_clause_for_all_goal() {
    solve_all(vec![clause!(forall(1) (apply "foo" (bound 0)))],
              goal!(forall(1) (apply "foo" (bound 0))),
              vec![r#"forall{?A -> "foo"(?A)}"#]);
}

#[test]
fn recursive() {
    // foo X :- foo X.
    //
    // Fails to prove `foo A`
    solve_all(vec![clause!(forall(1) (implies (apply "foo" (bound 0)) => (apply "foo" (bound 0))))],
              goal!(forall(1) (apply "foo" (bound 0))),
              vec!["<<overflow>>"])
}

#[test]
fn simple_impl() {
    solve_all(vec![
        // Copy implementedFor: i32
        clause!(apply "implementedFor" (apply "Copy") (apply "i32")),

        // Copy implementedFor: u32
        clause!(apply "implementedFor" (apply "Copy") (apply "u32")),

        // Copy implementedFor: Vec[?T] :- Copy implementedFor: ?T.
        clause!(forall(1) (implies
                           (apply "implementedFor" (apply "Copy") (bound 0)) =>
                           (apply "implementedFor" (apply "Copy") (apply "Vec" (bound 0)))))
    ],
              goal!(apply "implementedFor" (apply "Copy") (apply "Vec" (apply "i32"))),
              vec![r#""implementedFor"("Copy", "Vec"("i32"))"#]);
}

#[test]
fn conditional_impl() {
    let clauses = || {
        vec![
        // Copy implementedFor: i32
        clause!(apply "implementedFor" (apply "Copy") (apply "i32")),

        // Copy implementedFor: u32
        clause!(apply "implementedFor" (apply "Copy") (apply "u32")),

        // Trait implementedFor: Vec[?T] :- Copy implementedFor: ?T.
        clause!(forall(1) (implies
                           (apply "implementedFor" (apply "Copy") (bound 0)) =>
                           (apply "implementedFor" (apply "Trait") (apply "Vec" (bound 0))))),

        // Trait implementedFor: Vec[String].
        //
        // Key point: `String: Copy` does not hold.
        clause!(apply "implementedFor" (apply "Trait") (apply "Vec" (apply "String"))),

        // equate(A, A).
        clause!(forall(1) (apply "equate" (bound 0) (bound 0))),
    ]
    };

    solve_all(clauses(),
              goal!(apply "implementedFor" (apply "Trait") (apply "Vec" (apply "i32"))),
              vec![r#""implementedFor"("Trait", "Vec"("i32"))"#]);

    solve(&[DepthFirstSearch, Rust],
          clauses(),
          goal!(apply "implementedFor" (apply "Trait") (apply "Vec" (apply "String"))),
          vec![r#""implementedFor"("Trait", "Vec"("String"))"#]);

    // if asked to solve `Trait implementedFor: Vec[?A]`, we fail to infer what `?A` is.
    solve_rust(clauses(),
               goal!(exists(1) (apply "implementedFor" (apply "Trait") (apply "Vec" (bound 0)))),
               vec![r#"<<ambiguous>>"#]);

    // In these two variations, the second rule, `equate(?0, i32)`,
    // allows us to eventually solve the first
    solve(&[DepthFirstSearch, Rust],
          clauses(),
          goal!(exists(1) (and
                           (apply "implementedFor" (apply "Trait") (apply "Vec" (bound 0)))
                           (apply "equate" (bound 0) (apply "i32")))),
          vec![r#"and("implementedFor"("Trait", "Vec"("i32")), "equate"("i32", "i32"))"#]);
    solve(&[DepthFirstSearch, Rust],
          clauses(),
          goal!(exists(1) (and
                         (apply "equate" (bound 0) (apply "i32"))
                         (apply "implementedFor" (apply "Trait") (apply "Vec" (bound 0))))),
          vec![r#"and("equate"("i32", "i32"), "implementedFor"("Trait", "Vec"("i32")))"#]);
}

#[test]
fn if_then_else_one() {
    let clauses = || {
        vec![
            clause!(forall(1) (apply "=" (bound 0) (bound 0))),

            clause!(forall(2) (implies
                               (if (apply "=" (bound 0) (bound 1))
                                then false
                                else true)
                               =>
                               (apply "!=" (bound 0) (bound 1)))),
        ]
    };

    solve_rust(clauses(),
               goal!(if (apply "=" (apply "X") (apply "Y")) then false else true),
               vec![r#"if {"="("X", "Y")} then {false} else {true}"#]);

    solve_rust(clauses(),
               goal!(apply "!=" (apply "X") (apply "Y")),
               vec![r#""!="("X", "Y")"#]);

    solve_rust(clauses(),
               goal!(exists(1) (apply "!=" (bound 0) (apply "X"))),
               vec![r#"<<ambiguous>>"#]);

    // FIXME? It's somewhat surprising that this is provable. But it
    // is true that there will never be a way to unify `?A` and `X`,
    // even though the underlying semantic predicate is clearly
    // false. So we have to be careful about understanding what `if {}
    // then {} else {}` means -- that is, "not provable" does not
    // imply "not true".
    solve_rust(clauses(),
               goal!(forall(1) (apply "!=" (bound 0) (apply "X"))),
               vec![r#"forall{?A -> "!="(?A, "X")}"#]);
}
