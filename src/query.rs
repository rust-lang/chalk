// https://crates.io/crates/salsa
// hello world https://github.com/salsa-rs/salsa/blob/master/examples/hello_world/main.rs

use crate::coherence::orphan;
use crate::error::ChalkError;
use crate::lowering::LowerProgram;
use crate::program::Program;
use crate::program_environment::ProgramEnvironment;
use crate::rules::wf;
use chalk_ir::tls;
use chalk_ir::TypeKindId;
use chalk_solve::solve::SolverChoice;
use std::sync::Arc;

#[salsa::query_group(Lowering)]
pub trait LoweringDatabase {
    #[salsa::input]
    fn program_text(&self) -> Arc<String>;

    #[salsa::input]
    fn solver_choice(&self) -> SolverChoice;

    /// The program IR before recording specialization priorities.
    /// Do not use this query directly.
    fn program_ir(&self) -> Result<Arc<Program>, ChalkError>;

    /// The lowered IR.
    fn lowered_program(&self) -> Result<Arc<Program>, ChalkError>;

    /// The lowered IR, with checks performed.
    fn checked_program(&self) -> Result<Arc<Program>, ChalkError>;

    /// The program as logic.
    fn environment(&self) -> Result<Arc<ProgramEnvironment>, ChalkError>;
}

fn program_ir(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let text = db.program_text();
    Ok(Arc::new(chalk_parse::parse_program(&text)?.lower()?))
}

fn lowered_program(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let mut program = db.program_ir()?;
    let env = db.environment()?;

    Arc::make_mut(&mut program).record_specialization_priorities(env, db.solver_choice())?;

    Ok(program)
}

fn checked_program(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let program = db.lowered_program()?;
    let env = db.environment()?;

    orphan::perform_orphan_check(program.clone(), env.clone(), db.solver_choice())?;

    let () = tls::set_current_program(&program, || {
        let solver = wf::WfSolver {
            program: &*program,
            env: env,
            solver_choice: db.solver_choice(),
        };

        for (id, struct_datum) in &program.struct_data {
            if !solver.verify_struct_decl(struct_datum) {
                let name = program
                    .type_kinds
                    .get(&TypeKindId::StructId(*id))
                    .unwrap()
                    .name;
                return Err(wf::WfError::IllFormedTypeDecl(name));
            }
        }

        for impl_datum in program.impl_data.values() {
            if !solver.verify_trait_impl(impl_datum) {
                let trait_ref = impl_datum.binders.value.trait_ref.trait_ref();
                let name = program
                    .type_kinds
                    .get(&trait_ref.trait_id.into())
                    .unwrap()
                    .name;
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
