#![allow(unused_macros)]

use tracing_subscriber::{EnvFilter, FmtSubscriber};

macro_rules! lowering_success {
    (program $program:tt) => {
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        let result = $crate::test_util::with_tracing_logs(|| {
            chalk_integration::db::ChalkDatabase::with(
                &program_text[1..program_text.len() - 1],
                chalk_solve::SolverChoice::default(),
            )
            .checked_program()
        });
        if let Err(ref e) = result {
            println!("lowering error: {}", e);
        }
        assert!(result.is_ok());
    };
}

macro_rules! lowering_error {
    (program $program:tt error_msg { $expected:expr }) => {
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        let error = $crate::test_util::with_tracing_logs(|| {
            chalk_integration::db::ChalkDatabase::with(
                &program_text[1..program_text.len() - 1],
                chalk_solve::SolverChoice::default(),
            )
            .checked_program()
            .unwrap_err()
            .to_string()
        });
        let expected = $expected.to_string();
        crate::test_util::assert_same(&error, &expected);
    };
}

/// Run an action with a tracing log subscriber. The logging level is loaded
/// from `CHALK_DEBUG`.
pub fn with_tracing_logs<T>(action: impl FnOnce() -> T) -> T {
    let filter = EnvFilter::from_env("CHALK_DEBUG");
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_ansi(false)
        .without_time()
        .finish();
    tracing::subscriber::with_default(subscriber, action)
}

pub fn assert_same(result: &str, expected: &str) {
    println!("expected:\n{}", expected);
    println!("actual:\n{}", result);

    let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
    let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
    assert!(!expected1.is_empty(), "Expectation cannot be empty!");
    if !result1.starts_with(&expected1) {
        let prefix = &result1[..std::cmp::min(result1.len(), expected1.len())];
        // These will never be equal, which will cause a nice error message
        // to be displayed
        pretty_assertions::assert_eq!(expected1, prefix);
    }
}
