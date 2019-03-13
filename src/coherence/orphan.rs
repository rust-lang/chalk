use std::sync::Arc;

use super::CoherenceError;
use crate::rust_ir::*;
use chalk_ir::cast::*;
use chalk_ir::*;
use chalk_solve::ext::*;
use chalk_solve::solve::SolverChoice;
use failure::Fallible;

struct OrphanSolver {
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
}

pub(crate) fn perform_orphan_check(
    program: Arc<Program>,
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
) -> Fallible<()> {
    let solver = OrphanSolver { env, solver_choice };

    let local_impls = program
        .impl_data
        .values()
        // Only keep local impls (i.e. impls in the current crate)
        .filter(|impl_datum| impl_datum.binders.value.impl_type == ImplType::Local);

    for impl_datum in local_impls {
        if !solver.orphan_check(impl_datum) {
            let trait_id = impl_datum.binders.value.trait_ref.trait_ref().trait_id;
            let trait_id = program.type_kinds.get(&trait_id.into()).unwrap().name;
            Err(CoherenceError::FailedOrphanCheck(trait_id))?;
        }
    }

    Ok(())
}

impl OrphanSolver {
    // Test if a local impl violates the orphan rules.
    //
    // For `impl<T> Trait for MyType<T>` we generate:
    //
    //     forall<T> { LocalImplAllowed(MyType<T>: Trait) }
    //
    // This must be provable in order to pass the orphan check.
    fn orphan_check(&self, impl_datum: &ImplDatum) -> bool {
        debug_heading!("orphan_check(impl={:#?})", impl_datum);

        let impl_allowed: Goal = impl_datum
            .binders
            .map_ref(|bound_impl| {
                // Ignoring the polarization of the impl's polarized trait ref
                DomainGoal::LocalImplAllowed(bound_impl.trait_ref.trait_ref().clone())
            })
            .cast();

        let canonical_goal = &impl_allowed.into_closed_goal();
        let result = self
            .solver_choice
            .solve_root_goal(&self.env, canonical_goal)
            .unwrap()
            .is_some();
        debug!("overlaps: result = {:?}", result);
        result
    }
}
