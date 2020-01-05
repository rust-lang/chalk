#![allow(unused_macros)]

macro_rules! lowering_success {
    (program $program:tt) => {
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        let result = chalk_integration::db::ChalkDatabase::with(
            &program_text[1..program_text.len() - 1],
            chalk_solve::SolverChoice::default(),
        )
        .checked_program();
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
        let error = chalk_integration::db::ChalkDatabase::with(
            &program_text[1..program_text.len() - 1],
            chalk_solve::SolverChoice::default(),
        )
        .checked_program()
        .unwrap_err();
        let expected = $expected;
        assert_eq!(error.to_string(), expected.to_string());
    };
}
