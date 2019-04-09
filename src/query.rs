// https://crates.io/crates/salsa
// hello world https://github.com/salsa-rs/salsa/blob/master/examples/hello_world/main.rs

use crate::coherence::orphan;
use crate::coherence::SpecializationPriorities;
use crate::error::ChalkError;
use crate::lowering::LowerProgram;
use crate::program::Program;
use crate::program_environment::ProgramEnvironment;
use crate::rules::wf;
use crate::rules::RustIrSource;
use chalk_ir::tls;
use chalk_solve::ProgramClauseSet;
use chalk_solve::SolverChoice;
use std::sync::Arc;

#[salsa::query_group(Lowering)]
pub trait LoweringDatabase: ProgramClauseSet {
    #[salsa::input]
    fn program_text(&self) -> Arc<String>;

    #[salsa::input]
    fn solver_choice(&self) -> SolverChoice;

    fn program_ir(&self) -> Result<Arc<Program>, ChalkError>;

    /// Performs coherence check and computes which impls specialize
    /// one another (the "specialization priorities").
    fn coherence(&self) -> Result<Arc<SpecializationPriorities>, ChalkError>;

    fn orphan_check(&self) -> Result<(), ChalkError>;

    /// The lowered IR, with coherence, orphan, and WF checks performed.
    fn checked_program(&self) -> Result<Arc<Program>, ChalkError>;

    /// The program as logic.
    fn environment(&self) -> Result<Arc<ProgramEnvironment>, ChalkError>;
}

fn program_ir(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let text = db.program_text();
    Ok(Arc::new(chalk_parse::parse_program(&text)?.lower()?))
}

fn orphan_check(db: &impl LoweringDatabase) -> Result<(), ChalkError> {
    let program = db.program_ir()?;
    let solver_choice = db.solver_choice();

    tls::set_current_program(&program, || -> Result<(), ChalkError> {
        let local_impls = program.local_impl_ids();
        for impl_id in local_impls {
            orphan::perform_orphan_check(&*program, db, solver_choice, impl_id)?;
        }
        Ok(())
    })
}

fn coherence(db: &impl LoweringDatabase) -> Result<Arc<SpecializationPriorities>, ChalkError> {
    let program = db.program_ir()?;
    let priorities = program.specialization_priorities(db, db.solver_choice())?;
    let () = db.orphan_check()?;
    Ok(priorities)
}

fn checked_program(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let program = db.program_ir()?;

    db.coherence()?;

    let () = tls::set_current_program(&program, || {
        let solver = wf::WfSolver {
            program: &*program,
            env: db,
            solver_choice: db.solver_choice(),
        };

        for (&id, struct_datum) in &program.struct_data {
            if !solver.verify_struct_decl(struct_datum) {
                let name = program.type_name(id.into());
                return Err(wf::WfError::IllFormedTypeDecl(name));
            }
        }

        for impl_datum in program.impl_data.values() {
            if !solver.verify_trait_impl(impl_datum) {
                let trait_ref = impl_datum.binders.value.trait_ref.trait_ref();
                let name = program.type_name(trait_ref.trait_id.into());
                return Err(wf::WfError::IllFormedTraitImpl(name));
            }
        }

        Ok(())
    })?;

    Ok(program)
}

fn environment(db: &impl LoweringDatabase) -> Result<Arc<ProgramEnvironment>, ChalkError> {
    let env = db.program_ir()?.environment();
    Ok(Arc::new(env))
}
