#![cfg(test)]

use diff;
use chalk_parse;
use itertools::Itertools;
use std::fmt::Write;
use ir::lowering::{LowerProgram, LowerGoal};
use ir::{Goal, Program};
use solve::SolverChoice;
use errors::Result;

pub fn parse_and_lower_program(text: &str, solver_choice: SolverChoice) -> Result<Program> {
    chalk_parse::parse_program(text)?.lower(solver_choice)
}

pub fn parse_and_lower_goal(program: &Program, text: &str) -> Result<Box<Goal>> {
    chalk_parse::parse_goal(text)?.lower(program)
}

macro_rules! lowering_success {
    (program $program:tt) => {
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        let result = parse_and_lower_program(
            &program_text[1..program_text.len()-1],
            $crate::solve::SolverChoice::default()
        );
        if let Err(ref e) = result {
            println!("lowering error: {}", e);
        }
        assert!(
            result.is_ok()
        );
    }
}

macro_rules! lowering_error {
    (program $program:tt error_msg { $expected:expr }) => {
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        let error = parse_and_lower_program(
            &program_text[1..program_text.len()-1],
            $crate::solve::SolverChoice::default()
        ).unwrap_err();
        let expected = $crate::errors::Error::from($expected);
        assert_eq!(
            error.to_string(),
            expected.to_string()
        );
    }
}

crate fn assert_test_result_eq(expected: &str, actual: &str) {
    let expected_trimmed: String = expected
        .lines()
        .map(|l| l.trim())
        .intersperse("\n")
        .collect();

    let actual_trimmed: String = actual
        .lines()
        .map(|l| l.trim())
        .intersperse("\n")
        .collect();

    if expected_trimmed == actual_trimmed {
        return;
    }

    println!("expected:\n{}", expected);
    println!("actual:\n{}", actual);

    let diff = diff::lines(
        &expected_trimmed,
        &actual_trimmed,
    );

    // Skip to the first error:
    let diff = diff.iter().skip_while(|r| match r {
        diff::Result::Both(..) => true,
        _ => false,
    });

    let mut final_diff = String::new();
    let mut accumulator = vec![];
    for result in diff {
        let (prefix, s) = match result {
            diff::Result::Both(a, _b) => {
                // When we see things that are the same, don't print
                // them right away; wait until we see another line of
                // diff.
                accumulator.push(a);
                continue;
            }
            diff::Result::Left(a) => ("- ", a),
            diff::Result::Right(a) => ("+ ", a),
        };

        for l in accumulator.drain(..) {
            writeln!(&mut final_diff, "  {}", l).unwrap();
        }

        writeln!(&mut final_diff, "{}{}", prefix, s).unwrap();
    }

    assert!(false, "expected did not match actual, diff:\n{}", final_diff);
}
