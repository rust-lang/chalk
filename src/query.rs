// https://crates.io/crates/salsa
// hello world https://github.com/salsa-rs/salsa/blob/master/examples/hello_world/main.rs

use crate::error::ChalkError;
use crate::lowering::LowerProgram;
use crate::program::Program;
use crate::program_environment::ProgramEnvironment;
use chalk_ir::tls;
use chalk_ir::TraitId;
use chalk_rules::clauses::ToProgramClauses;
use chalk_rules::coherence::orphan;
use chalk_rules::coherence::{CoherenceSolver, SpecializationPriorities};
use chalk_rules::wf;
use chalk_rules::ChalkRulesDatabase;
use chalk_rules::RustIrSource;
use chalk_solve::ChalkSolveDatabase;
use chalk_solve::Solver;
use chalk_solve::SolverChoice;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

#[salsa::query_group(Lowering)]
pub trait LoweringDatabase: ChalkRulesDatabase + ChalkSolveDatabase + RustIrSource {
    #[salsa::input]
    fn program_text(&self) -> Arc<String>;

    #[salsa::input]
    fn solver_choice(&self) -> SolverChoice;

    fn program_ir(&self) -> Result<Arc<Program>, ChalkError>;

    /// Performs coherence check and computes which impls specialize
    /// one another (the "specialization priorities").
    fn coherence(&self) -> Result<BTreeMap<TraitId, Arc<SpecializationPriorities>>, ChalkError>;

    fn orphan_check(&self) -> Result<(), ChalkError>;

    /// The lowered IR, with coherence, orphan, and WF checks performed.
    fn checked_program(&self) -> Result<Arc<Program>, ChalkError>;

    /// The program as logic.
    fn environment(&self) -> Result<Arc<ProgramEnvironment>, ChalkError>;

    /// Creates the solver we can use to solve goals. This solver
    /// stores intermediate, cached state, which is why it is behind a
    /// mutex. Moreover, if the set of program clauses change, that
    /// cached state becomes invalid, so the query is marked as
    /// volatile, thus ensuring that the solver is recreated in every
    /// revision (i.e., each time source program changes).
    #[salsa::volatile]
    fn solver(&self) -> Arc<Mutex<Solver>>;
}

fn program_ir(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let text = db.program_text();
    Ok(Arc::new(chalk_parse::parse_program(&text)?.lower()?))
}

fn orphan_check(db: &impl LoweringDatabase) -> Result<(), ChalkError> {
    let program = db.program_ir()?;

    tls::set_current_program(&program, || -> Result<(), ChalkError> {
        let local_impls = program.local_impl_ids();
        for impl_id in local_impls {
            orphan::perform_orphan_check(db, impl_id)?;
        }
        Ok(())
    })
}

fn coherence(
    db: &impl LoweringDatabase,
) -> Result<BTreeMap<TraitId, Arc<SpecializationPriorities>>, ChalkError> {
    let program = db.program_ir()?;

    let priorities_map: Result<BTreeMap<_, _>, ChalkError> = program
        .trait_data
        .keys()
        .map(|&trait_id| {
            let solver = CoherenceSolver::new(db, trait_id);
            let priorities = solver.specialization_priorities()?;
            Ok((trait_id, priorities))
        })
        .collect();
    let priorities_map = priorities_map?;

    let () = db.orphan_check()?;

    Ok(priorities_map)
}

fn checked_program(db: &impl LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let program = db.program_ir()?;

    db.coherence()?;

    let () = tls::set_current_program(&program, || -> Result<(), ChalkError> {
        let solver = wf::WfSolver::new(db);

        for &id in program.struct_data.keys() {
            solver.verify_struct_decl(id)?;
        }

        for &impl_id in program.impl_data.keys() {
            solver.verify_trait_impl(impl_id)?;
        }

        Ok(())
    })?;

    Ok(program)
}

fn environment(db: &impl LoweringDatabase) -> Result<Arc<ProgramEnvironment>, ChalkError> {
    let program = db.program_ir()?;

    // Construct the set of *clauses*; these are sort of a compiled form
    // of the data above that always has the form:
    //
    //       forall P0...Pn. Something :- Conditions
    let mut program_clauses = program.custom_clauses.clone();

    program
        .associated_ty_data
        .values()
        .for_each(|d| d.to_program_clauses(db, &mut program_clauses));

    program
        .trait_data
        .values()
        .for_each(|d| d.to_program_clauses(db, &mut program_clauses));

    program
        .struct_data
        .values()
        .for_each(|d| d.to_program_clauses(db, &mut program_clauses));

    for (&auto_trait_id, auto_trait) in program
        .trait_data
        .iter()
        .filter(|(_, auto_trait)| auto_trait.binders.value.flags.auto)
    {
        for (&struct_id, struct_datum) in program.struct_data.iter() {
            chalk_rules::clauses::push_auto_trait_impls(
                auto_trait_id,
                auto_trait,
                struct_id,
                struct_datum,
                db,
                &mut program_clauses,
            );
        }
    }

    for datum in program.impl_data.values() {
        // If we encounter a negative impl, do not generate any rule. Negative impls
        // are currently just there to deactivate default impls for auto traits.
        if datum.binders.value.trait_ref.is_positive() {
            datum.to_program_clauses(db, &mut program_clauses);
            datum
                .binders
                .value
                .associated_ty_values
                .iter()
                .for_each(|atv| atv.to_program_clauses(db, &mut program_clauses));
        }
    }

    Ok(Arc::new(ProgramEnvironment::new(program_clauses)))
}

fn solver(db: &impl LoweringDatabase) -> Arc<Mutex<Solver>> {
    let choice = db.solver_choice();
    Arc::new(Mutex::new(choice.into_solver()))
}
