#![allow(unused_macros)]

macro_rules! lower_program_with_db {
    (program $program:tt database $database:ty) => {{
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        <$database>::with(
            &program_text[1..program_text.len() - 1],
            chalk_solve::SolverChoice::default(),
        )
    }};
}

macro_rules! lower_goal {
    (goal $goal:tt program $program:expr) => {{
        let goal_text = stringify!($goal);
        assert!(goal_text.starts_with("{"));
        assert!(goal_text.ends_with("}"));
        chalk_parse::parse_goal(&goal_text[1..goal_text.len() - 1])
            .unwrap()
            .lower($program)
            .unwrap()
    }};
}

macro_rules! lowering_success {
    (program $program:tt) => {
        let result = lower_program_with_db!(
            program $program
            database chalk_integration::db::ChalkDatabase
        ).checked_program();
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
