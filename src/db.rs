#![allow(non_camel_case_types)]

use crate::query::{self, ProgramSolverChoice, ProgramText};
use chalk_solve::solve::SolverChoice;
use salsa::Database;
use std::sync::Arc;

#[derive(Default)]
pub struct ChalkDatabase {
    runtime: salsa::Runtime<ChalkDatabase>,
}

impl Database for ChalkDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime<ChalkDatabase> {
        &self.runtime
    }
}

impl ChalkDatabase {
    pub fn with_program<F: FnOnce(&mut ChalkDatabase) -> R, R>(
        program_text: Arc<String>,
        solver_choice: SolverChoice,
        f: F,
    ) -> R {
        let mut db = ChalkDatabase::default();

        db.query_mut(ProgramText).set((), program_text);
        db.query_mut(ProgramSolverChoice).set((), solver_choice);

        f(&mut db)
    }
}

salsa::database_storage! {
    pub struct DatabaseStorage for ChalkDatabase {
        impl query::LoweringDatabase {
            fn program_text() for query::ProgramText;
            fn solver_choice() for query::ProgramSolverChoice;
            fn program_ir() for query::ProgramIr;
            fn lowered_program() for query::LoweredProgram;
            fn checked_program() for query::CheckedProgram;
            fn environment() for query::Environment;
        }
    }
}
