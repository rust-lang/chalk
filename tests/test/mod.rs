#![allow(non_snake_case)]

use chalk_integration::db::ChalkDatabase;
use chalk_integration::lowering::LowerGoal;
use chalk_integration::query::LoweringDatabase;
use chalk_ir;
use chalk_ir::interner::ChalkIr;
use chalk_solve::ext::*;
use chalk_solve::RustIrDatabase;
use chalk_solve::{Solution, SolverChoice};

#[cfg(feature = "bench")]
mod bench;
mod coherence;
mod wf_lowering;

fn assert_result(mut result: Option<Solution<ChalkIr>>, expected: &str) {
    // sort constraints, since the different solvers may output them in different order
    match &mut result {
        Some(Solution::Unique(solution)) => {
            solution
                .value
                .constraints
                .sort_by_key(|c| format!("{:?}", c));
        }
        _ => {}
    }
    let result = match result {
        Some(v) => format!("{}", v),
        None => format!("No possible solution"),
    };

    assert_same(&result, expected);
}

fn assert_same(result: &str, expected: &str) {
    println!("expected:\n{}", expected);
    println!("actual:\n{}", result);

    let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
    let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
    assert!(!expected1.is_empty() && result1.starts_with(&expected1));
}

// different goals
enum TestGoal {
    // solver should produce same aggregated single solution
    Aggregated(&'static str),
    // solver should produce exactly multiple solutions
    All(Vec<&'static str>),
    // solver should produce first same multiple solutions
    First(Vec<&'static str>),
}

macro_rules! test {
    (program $program:tt $($goals:tt)*) => {
        test!(@program[$program]
              @parsed_goals[]
              @unparsed_goals[$($goals)*])
    };

    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[]) => {
        solve_goal(stringify!($program), vec![$($parsed_goals),*])
    };

    // goal { G } yields { "Y" } -- test both solvers behave the same (the default)
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields { $expected:expr }
        $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), SolverChoice::default(), TestGoal::Aggregated($expected))
              ]
              @unparsed_goals[$($unparsed_goals)*])
    };

    // goal { G } yields_all { "Y1", "Y2", ... , "YN" } -- test both solvers gets exactly N same answers in
    // the same order
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields_all { $($expected:expr),* }
        $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), SolverChoice::default(), TestGoal::All(vec![$($expected),*]))
              ]
              @unparsed_goals[$($unparsed_goals)*])
    };

    // goal { G } yields_first { "Y1", "Y2", ... , "YN" } -- test both solvers gets at least N same first answers
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields_first { $($expected:expr),* }
        $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), SolverChoice::default(), TestGoal::First(vec![$($expected),*]))
              ]
              @unparsed_goals[$($unparsed_goals)*])
    };

    // goal { G } yields[C1] { "Y1" } yields[C2] { "Y2" } -- test that solver C1 yields Y1
    // and C2 yields Y2
    //
    // Annoyingly, to avoid getting a parsing ambiguity error, we have
    // to distinguish the case where there are other goals to come
    // (this rule) for the last goal in the list (next rule). There
    // might be a more elegant fix than copy-and-paste but this works.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields[$C:expr] { $expected:expr }
            goal $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), $C, TestGoal::Aggregated($expected))
              ]
              @unparsed_goals[goal $($unparsed_goals)*])
    };

    // same as above, but for the final goal in the list.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields[$C:expr] { $expected:expr }
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), $C, TestGoal::Aggregated($expected))
                ]
              @unparsed_goals[])
    };

    // goal { G } yields_all[C1] { "Y1" } yields_all[C2] { "Y2" } -- test that solver C1 yields Y1
    // and C2 yields Y2
    //
    // Annoyingly, to avoid getting a parsing ambiguity error, we have
    // to distinguish the case where there are other goals to come
    // (this rule) for the last goal in the list (next rule). There
    // might be a more elegant fix than copy-and-paste but this works.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields_all[$C:expr] { $($expected:expr),* }
            goal $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), $C, TestGoal::All(vec![$($expected),*]))
              ]
              @unparsed_goals[goal $($unparsed_goals)*])
    };

    // same as above, but for the final goal in the list.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields_all[$C:expr] { $($expected:expr),* }
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), $C, TestGoal::All(vec![$($expected),*]))
                ]
              @unparsed_goals[])
    };

    // goal { G } yields_first[C1] { "Y1" } yields_first[C2] { "Y2" } -- test that solver C1 yields Y1
    // and C2 yields Y2
    //
    // Annoyingly, to avoid getting a parsing ambiguity error, we have
    // to distinguish the case where there are other goals to come
    // (this rule) for the last goal in the list (next rule). There
    // might be a more elegant fix than copy-and-paste but this works.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields_first[$C:expr] { $($expected:expr),* }
            goal $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), $C, TestGoal::First(vec![$($expected),*]))
              ]
              @unparsed_goals[goal $($unparsed_goals)*])
    };

    // same as above, but for the final goal in the list.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields_first[$C:expr] { $($expected:expr),* }
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), $C, TestGoal::First(vec![$($expected),*]))
                ]
              @unparsed_goals[])
    };
}

fn solve_goal(program_text: &str, goals: Vec<(&str, SolverChoice, TestGoal)>) {
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));

    let mut db = ChalkDatabase::with(
        &program_text[1..program_text.len() - 1],
        SolverChoice::default(),
    );

    let program = db.checked_program().unwrap();

    for (goal_text, solver_choice, expected) in goals {
        match (&solver_choice, &expected) {
            (SolverChoice::Recursive { .. }, TestGoal::All(_))
            | (SolverChoice::Recursive { .. }, TestGoal::First(_)) => {
                println!(
                    "skipping goal {} for recursive solver because it requires solve_multiple",
                    goal_text
                );
                continue;
            }
            _ => {}
        };

        if db.solver_choice() != solver_choice {
            db.set_solver_choice(solver_choice);
        }

        chalk_ir::tls::set_current_program(&program, || {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = chalk_parse::parse_goal(&goal_text[1..goal_text.len() - 1])
                .unwrap()
                .lower(&*program)
                .unwrap();

            println!("using solver: {:?}", solver_choice);
            let peeled_goal = goal.into_peeled_goal(db.interner());
            match expected {
                TestGoal::Aggregated(expected) => {
                    let result = db.solve(&peeled_goal);
                    assert_result(result, expected);
                }
                TestGoal::All(expected) => {
                    let mut expected = expected.into_iter();
                    assert!(
                        db.solve_multiple(&peeled_goal, |result, next_result| {
                            match expected.next() {
                                Some(expected) => {
                                    assert_same(&format!("{}", &result), expected);
                                }
                                None => {
                                    assert!(!next_result, "Unexpected next solution");
                                }
                            }
                            true
                        }),
                        "Not all solutions processed"
                    );
                    if expected.next().is_some() {
                        panic!("Not all solutions processed");
                    }
                }
                TestGoal::First(expected) => {
                    let mut expected = expected.into_iter();
                    db.solve_multiple(&peeled_goal, |result, next_result| match expected.next() {
                        Some(solution) => {
                            assert_same(&format!("{}", &result), solution);
                            if !next_result {
                                assert!(expected.next().is_none(), "Not enough solutions found");
                            }
                            true
                        }
                        None => false,
                    });
                    assert!(expected.next().is_none(), "Not enough solutions found");
                }
            }
        });
    }
}

mod auto_traits;
mod coherence_goals;
mod coinduction;
mod cycle;
mod existential_types;
mod implied_bounds;
mod impls;
mod misc;
mod negation;
mod projection;
mod unify;
mod wf_goals;
