#![allow(non_camel_case_types)]

use crate::query::{Lowering, LoweringDatabase};
use chalk_solve::solve::SolverChoice;
use salsa::Database;
use std::sync::Arc;

#[salsa::database(Lowering)]
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
    pub fn with(
        program_text: &str,
        solver_choice: SolverChoice
    ) -> Self {
        let mut db = ChalkDatabase::default();
        db.set_program_text(Arc::new(program_text.to_string()));
        db.set_solver_choice(solver_choice);
        db
    }

    pub fn with_program<F: FnOnce(&mut ChalkDatabase) -> R, R>(
        program_text: Arc<String>,
        solver_choice: SolverChoice,
        f: F,
    ) -> R {
        let mut db = ChalkDatabase::default();

        db.set_program_text(program_text);
        db.set_solver_choice(solver_choice);

        f(&mut db)
    }
}
