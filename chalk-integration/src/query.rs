// https://crates.io/crates/salsa
// hello world https://github.com/salsa-rs/salsa/blob/master/examples/hello_world/main.rs

use crate::error::ChalkError;
use crate::interner::ChalkIr;
use crate::lowering::Lower;
use crate::program::Program;
use crate::program_environment::ProgramEnvironment;
use crate::tls;
use crate::SolverChoice;
use chalk_ir::TraitId;
use chalk_solve::clauses::builder::ClauseBuilder;
use chalk_solve::clauses::program_clauses::ToProgramClauses;
use chalk_solve::coherence::orphan;
use chalk_solve::coherence::{CoherenceSolver, SpecializationPriorities};
use chalk_solve::wf;
use chalk_solve::RustIrDatabase;
use chalk_solve::Solver;
use salsa::Database;
use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::Mutex;

#[salsa::query_group(Lowering)]
pub trait LoweringDatabase:
    RustIrDatabase<ChalkIr> + Database + Upcast<dyn RustIrDatabase<ChalkIr>>
{
    #[salsa::input]
    fn program_text(&self) -> Arc<String>;

    #[salsa::input]
    fn solver_choice(&self) -> SolverChoice;

    fn program_ir(&self) -> Result<Arc<Program>, ChalkError>;

    /// Performs coherence check and computes which impls specialize
    /// one another (the "specialization priorities").
    fn coherence(
        &self,
    ) -> Result<BTreeMap<TraitId<ChalkIr>, Arc<SpecializationPriorities<ChalkIr>>>, ChalkError>;

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
    // HACK: salsa requires that queries return types that implement `Eq`
    fn solver(&self) -> ArcEq<Mutex<Box<dyn Solver<ChalkIr>>>>;
}

// Needed to go from dyn LoweringDatabase -> dyn RustIrDatabase
// These traits are basically vendored (slightly modified) from https://github.com/connicpu/upcast
pub trait Upcast<U: ?Sized> {
    fn upcast(&self) -> &U;
}

pub trait UpcastFrom<T: ?Sized> {
    fn upcast_from(val: &T) -> &Self;
}

impl<'a, T: RustIrDatabase<ChalkIr> + 'a> UpcastFrom<T> for dyn RustIrDatabase<ChalkIr> + 'a {
    fn upcast_from(val: &T) -> &(dyn RustIrDatabase<ChalkIr> + 'a) {
        val
    }
}

impl<T: ?Sized, U: ?Sized> Upcast<U> for T
where
    U: UpcastFrom<T>,
{
    fn upcast(&self) -> &U {
        U::upcast_from(self)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ArcEq<T>(Arc<T>);

impl<T> ArcEq<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> Deref for ArcEq<T> {
    type Target = Arc<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ArcEq<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> PartialEq<ArcEq<T>> for ArcEq<T> {
    fn eq(&self, other: &ArcEq<T>) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for ArcEq<T> {}

impl<T> Clone for ArcEq<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

fn program_ir(db: &dyn LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let text = db.program_text();
    Ok(Arc::new(chalk_parse::parse_program(&text)?.lower()?))
}

fn orphan_check(db: &dyn LoweringDatabase) -> Result<(), ChalkError> {
    let program = db.program_ir()?;

    tls::set_current_program(&program, || -> Result<(), ChalkError> {
        let local_impls = program.local_impl_ids();
        for impl_id in local_impls {
            let mut solver = db.solver_choice().into_solver();
            orphan::perform_orphan_check::<ChalkIr>(db.upcast(), &mut *solver, impl_id)?;
        }
        Ok(())
    })
}

fn coherence(
    db: &dyn LoweringDatabase,
) -> Result<BTreeMap<TraitId<ChalkIr>, Arc<SpecializationPriorities<ChalkIr>>>, ChalkError> {
    let program = db.program_ir()?;
    let solver_choice = db.solver_choice();
    let priorities_map = tls::set_current_program(&program, || -> Result<_, ChalkError> {
        let solver_builder = || solver_choice.into_solver();
        let priorities_map: Result<BTreeMap<_, _>, ChalkError> = program
            .trait_data
            .keys()
            .map(|&trait_id| {
                let solver: CoherenceSolver<ChalkIr> =
                    CoherenceSolver::new(db.upcast(), &solver_builder, trait_id);
                let priorities = solver.specialization_priorities()?;
                Ok((trait_id, priorities))
            })
            .collect();
        let priorities_map = priorities_map?;
        Ok(priorities_map)
    });
    let () = db.orphan_check()?;
    priorities_map
}

fn checked_program(db: &dyn LoweringDatabase) -> Result<Arc<Program>, ChalkError> {
    let program = db.program_ir()?;

    db.coherence()?;

    let solver_choice = db.solver_choice();
    let () = tls::set_current_program(&program, || -> Result<(), ChalkError> {
        let solver_builder = || solver_choice.into_solver();
        let solver: wf::WfSolver<ChalkIr> = wf::WfSolver::new(db.upcast(), &solver_builder);
        for &id in program.adt_data.keys() {
            solver.verify_adt_decl(id)?;
        }

        for &opaque_ty_id in program.opaque_ty_data.keys() {
            solver.verify_opaque_ty_decl(opaque_ty_id)?;
        }

        for &impl_id in program.impl_data.keys() {
            solver.verify_trait_impl(impl_id)?;
        }

        Ok(())
    })?;

    Ok(program)
}

fn environment(db: &dyn LoweringDatabase) -> Result<Arc<ProgramEnvironment>, ChalkError> {
    let program = db.program_ir()?;

    // Construct the set of *clauses*; these are sort of a compiled form
    // of the data above that always has the form:
    //
    //       forall P0...Pn. Something :- Conditions
    let mut program_clauses = program.custom_clauses.clone();

    let builder = &mut ClauseBuilder::new(db.upcast(), &mut program_clauses);

    let env = chalk_ir::Environment::new(builder.interner());

    program
        .associated_ty_data
        .values()
        .for_each(|d| d.to_program_clauses(builder, &env));

    program
        .trait_data
        .values()
        .for_each(|d| d.to_program_clauses(builder, &env));

    program
        .adt_data
        .values()
        .for_each(|d| d.to_program_clauses(builder, &env));

    for (&auto_trait_id, _) in program
        .trait_data
        .iter()
        .filter(|(_, auto_trait)| auto_trait.is_auto_trait())
    {
        for adt_datum in program.adt_data.values() {
            builder.push_binders(adt_datum.binders.clone(), |builder, _| {
                let ty = chalk_ir::TyKind::Adt(adt_datum.id, builder.substitution_in_scope());
                chalk_solve::clauses::push_auto_trait_impls(builder, auto_trait_id, &ty)
                    .map_err(|_| ())
                    .unwrap();
            });
        }
    }

    for datum in program.impl_data.values() {
        // If we encounter a negative impl, do not generate any rule. Negative impls
        // are currently just there to deactivate default impls for auto traits.
        if datum.is_positive() {
            datum.to_program_clauses(builder, &env);
            datum
                .associated_ty_value_ids
                .iter()
                .map(|&atv_id| db.associated_ty_value(atv_id))
                .for_each(|atv| atv.to_program_clauses(builder, &env));
        }
    }

    Ok(Arc::new(ProgramEnvironment::new(program_clauses)))
}

fn solver(db: &dyn LoweringDatabase) -> ArcEq<Mutex<Box<dyn Solver<ChalkIr>>>> {
    db.salsa_runtime().report_untracked_read();
    let choice = db.solver_choice();
    ArcEq::new(Mutex::new(choice.into_solver()))
}
