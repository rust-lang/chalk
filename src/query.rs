// https://crates.io/crates/salsa
// hello world https://github.com/salsa-rs/salsa/blob/master/examples/hello_world/main.rs

use crate::rust_ir;
use crate::rust_ir::lowering::LowerProgram;
use chalk_solve::solve::SolverChoice;
use std::sync::Arc;

salsa::query_group! {
    pub trait LoweringDatabase: salsa::Database {
        fn program_text() -> Arc<String> {
            type ProgramText;

            storage input;
        }

        fn solver_choice() -> SolverChoice {
            type ProgramSolverChoice;

            storage input;
        }

        // FIXME: Result<..., String> is only needed because the error type is not clone

        /// The lowered IR.
        fn lowered_program() -> Result<Arc<rust_ir::Program>, String> {
            type LoweredProgram;
        }

        /// The lowered IR, with checks performed.
        fn checked_program() -> Result<Arc<rust_ir::Program>, String> {
            type CheckedProgram;
        }
    }
}

fn lowered_program(db: &impl LoweringDatabase) -> Result<Arc<rust_ir::Program>, String> {
    let x: crate::errors::Result<_> = try {
        let text = db.program_text();
        Arc::new(chalk_parse::parse_program(&text)?.lower(db.solver_choice())?)
    };

    x.map_err(|err| err.to_string())
}

fn checked_program(db: &impl LoweringDatabase) -> Result<Arc<rust_ir::Program>, String> {
    db.lowered_program().and_then(|program| {
        program
            .perform_checks(db.solver_choice())
            .map_err(|err| err.to_string())?;
        Ok(program)
    })
}
