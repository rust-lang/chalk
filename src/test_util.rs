#![cfg(test)]

use diff;
use itertools::Itertools;
use std::fmt::Write;

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
