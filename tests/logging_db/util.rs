//! Macros / utilities for logging_db tests.
//!
//! This is not a submodule of `test_util` as it depends on macros declared in
//! `test/mod.rs`, and `test_util.rs` is compiled both with and without access
//! to `test/`. We can't compile without access to `test/`, so we can't be under
//! of `test_util.rs`.
use chalk_integration::{
    db::ChalkDatabase, lowering::lower_goal, program::Program, query::LoweringDatabase,
    SolverChoice,
};
use chalk_solve::ext::*;
use chalk_solve::logging_db::LoggingRustIrDatabase;
use chalk_solve::RustIrDatabase;

use crate::test::assert_result_str;

type TestGoal = crate::test::TestGoal<&'static str>;

macro_rules! logging_db_output_sufficient {
    ($($arg:tt)*) => {{
        use chalk_integration::SolverChoice;
        use crate::test::*;
        let (program, goals) = parse_test_data!($($arg)*);
        crate::logging_db::util::logging_db_output_sufficient(program, goals)
    }};
}

pub fn logging_db_output_sufficient(
    program_text: &str,
    goals: Vec<(&str, Vec<SolverChoice>, TestGoal)>,
) {
    println!("program {}", program_text);
    assert!(program_text.starts_with('{'));
    assert!(program_text.ends_with('}'));

    let goals = goals
        .iter()
        .flat_map(|(a, bs, c)| bs.iter().map(move |b| (a, b, c)));

    let output_text = {
        let db = ChalkDatabase::with(
            &program_text[1..program_text.len() - 1],
            SolverChoice::default(),
        );

        let program = db.program_ir().unwrap();
        let wrapped = LoggingRustIrDatabase::<_, Program, _>::new(program.clone());

        chalk_integration::tls::set_current_program(&program, || {
            for (goal_text, solver_choice, expected) in goals.clone() {
                let mut solver = solver_choice.into_solver();

                println!("----------------------------------------------------------------------");
                println!("---- first run on original test code ---------------------------------");
                println!("goal {}", goal_text);
                assert!(goal_text.starts_with('{'));
                assert!(goal_text.ends_with('}'));
                let goal = lower_goal(
                    &*chalk_parse::parse_goal(&goal_text[1..goal_text.len() - 1]).unwrap(),
                    &*program,
                )
                .unwrap();

                println!("using solver: {:?}", solver_choice);
                let peeled_goal = goal.into_peeled_goal(db.interner());
                match expected {
                    TestGoal::Aggregated(expected) => {
                        let result = solver.solve(&wrapped, &peeled_goal);
                        assert_result_str(result, expected, db.interner());
                    }
                    _ => panic!("only aggregated test goals supported for logger goals"),
                }
            }

            wrapped.to_string()
        })
    };

    println!("----------------------------------------------------------------------");
    println!("logging db output program:\n{}\n", output_text);

    let db = ChalkDatabase::with(&output_text, SolverChoice::default());

    // Note: we are explicitly not calling `.checked_program()`, as our output
    // is not intended to be well formed.
    let new_program = match db.program_ir() {
        Ok(v) => v,
        Err(e) => panic!("Error checking recreated chalk program: {}", e),
    };

    for (goal_text, solver_choice, expected) in goals {
        let mut solver = solver_choice.into_solver();

        chalk_integration::tls::set_current_program(&new_program, || {
            println!("----------------------------------------------------------------------");
            println!("---- second run on code output by logger -----------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with('{'));
            assert!(goal_text.ends_with('}'));
            let goal = lower_goal(
                &*chalk_parse::parse_goal(&goal_text[1..goal_text.len() - 1]).unwrap(),
                &*new_program,
            )
            .unwrap();

            println!("using solver: {:?}", solver_choice);
            let peeled_goal = goal.into_peeled_goal(db.interner());
            match expected {
                TestGoal::Aggregated(expected) => {
                    let result = solver.solve(&db, &peeled_goal);
                    assert_result_str(result, expected, db.interner());
                }
                _ => panic!("only aggregated test goals supported for logger goals"),
            }
        });
    }
}
