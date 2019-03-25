// https://crates.io/crates/salsa
// hello world https://github.com/salsa-rs/salsa/blob/master/examples/hello_world/main.rs

use crate::coherence::orphan;
use crate::rules::wf;
use crate::rust_ir;
use crate::rust_ir::lowering::LowerProgram;
use chalk_ir::ProgramEnvironment;
use chalk_solve::solve::SolverChoice;
use std::sync::Arc;

#[salsa::query_group(Lowering)]
pub trait LoweringDatabase {
    #[salsa::input]
    fn program_text(&self) -> Arc<String>;

    #[salsa::input]
    fn solver_choice(&self) -> SolverChoice;

    // FIXME: Result<..., String> is only needed because the error type is not clone

    /// The program IR before recording specialization priorities.
    /// Do not use this query directly.
    fn program_ir(&self) -> Result<Arc<rust_ir::Program>, String>;

    /// The lowered IR.
    fn lowered_program(&self) -> Result<Arc<rust_ir::Program>, String>;

    /// The lowered IR, with checks performed.
    fn checked_program(&self) -> Result<Arc<rust_ir::Program>, String>;

    /// The program as logic.
    fn environment(&self) -> Result<Arc<ProgramEnvironment>, String>;
}

fn program_ir(db: &impl LoweringDatabase) -> Result<Arc<rust_ir::Program>, String> {
    let text = db.program_text();
    let x = chalk_parse::parse_program(&text)
        .and_then(|p| p.lower())
        .map(Arc::new);

    x.map_err(|err| err.to_string())
}

fn lowered_program(db: &impl LoweringDatabase) -> Result<Arc<rust_ir::Program>, String> {
    let mut program = db.program_ir()?;
    let env = db.environment()?;

    Arc::make_mut(&mut program)
        .record_specialization_priorities(env, db.solver_choice())
        .map_err(|e| e.to_string())?;

    Ok(program)
}

fn checked_program(db: &impl LoweringDatabase) -> Result<Arc<rust_ir::Program>, String> {
    let program = db.lowered_program()?;
    let env = db.environment()?;

    orphan::perform_orphan_check(program.clone(), env.clone(), db.solver_choice())
        .map_err(|e| e.to_string())?;

    wf::verify_well_formedness(program.clone(), env, db.solver_choice())
        .map_err(|e| e.to_string())?;

    Ok(program)
}

fn environment(db: &impl LoweringDatabase) -> Result<Arc<ProgramEnvironment>, String> {
    let env = db.program_ir()?.environment();
    Ok(Arc::new(env))
}
