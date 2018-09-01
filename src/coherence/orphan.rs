use std::sync::Arc;

use errors::*;
use ir::*;
use rust_ir::*;
use cast::*;
use solve::SolverChoice;

struct OrphanSolver {
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
}

impl Program {
    crate fn perform_orphan_check(&self, solver_choice: SolverChoice) -> Result<()> {
        let solver = OrphanSolver {
            env: Arc::new(self.environment()),
            solver_choice,
        };

        let local_impls = self.impl_data
            .values()
            // Only keep local impls (i.e. impls in the current crate)
            .filter(|impl_datum| impl_datum.binders.value.impl_type == ImplType::Local);

        for impl_datum in local_impls {
            if !solver.orphan_check(impl_datum) {
                let trait_id = impl_datum.binders.value.trait_ref.trait_ref().trait_id;
                let trait_id = self.type_kinds.get(&trait_id).unwrap().name;
                return Err(Error::from_kind(ErrorKind::FailedOrphanCheck(trait_id)));
            }
        }

        Ok(())
    }
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

        let impl_allowed: Goal = impl_datum.binders.map_ref(|bound_impl| {
            // Ignoring the polarization of the impl's polarized trait ref
            DomainGoal::LocalImplAllowed(bound_impl.trait_ref.trait_ref().clone())
        }).cast();

        let canonical_goal = &impl_allowed.into_closed_goal();
        let result = self.solver_choice
            .solve_root_goal(&self.env, canonical_goal)
            .unwrap()
            .is_some();
        debug!("overlaps: result = {:?}", result);
        result
    }
}
