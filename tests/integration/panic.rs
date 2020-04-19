use chalk_integration::db::ChalkDatabase;
use chalk_integration::query::LoweringDatabase;
use chalk_ir::interner::ChalkIr;
use chalk_ir::AssocTypeId;
use chalk_ir::Goal;
use chalk_ir::ImplId;
use chalk_ir::InEnvironment;
use chalk_ir::OpaqueTyId;
use chalk_ir::Parameter;
use chalk_ir::ProgramClause;
use chalk_ir::StructId;
use chalk_ir::TraitId;
use chalk_ir::TypeName;
use chalk_ir::UCanonical;
use chalk_rust_ir::AssociatedTyDatum;
use chalk_rust_ir::AssociatedTyValue;
use chalk_rust_ir::AssociatedTyValueId;
use chalk_rust_ir::ImplDatum;
use chalk_rust_ir::OpaqueTyDatum;
use chalk_rust_ir::StructDatum;
use chalk_rust_ir::TraitDatum;
use chalk_rust_ir::WellKnownTrait;
use chalk_solve::RustIrDatabase;
use chalk_solve::Solution;
use chalk_solve::SolverChoice;
use std::sync::Arc;

#[derive(Debug, Default)]
struct MockDatabase {
    chalk_db: ChalkDatabase,
    panic: bool,
}

impl MockDatabase {
    pub fn with(program_text: &str, solver_choice: SolverChoice) -> Self {
        Self {
            chalk_db: ChalkDatabase::with(program_text, solver_choice),
            panic: false,
        }
    }

    pub fn solve(
        &self,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> Option<Solution<ChalkIr>> {
        let solver = self.chalk_db.solver();
        let solution = solver.lock().unwrap().solve(self, goal);
        solution
    }
}

impl RustIrDatabase<ChalkIr> for MockDatabase {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        if self.panic {
            unimplemented!()
        } else {
            self.chalk_db.custom_clauses()
        }
    }

    fn associated_ty_data(&self, ty: AssocTypeId<ChalkIr>) -> Arc<AssociatedTyDatum<ChalkIr>> {
        self.chalk_db.associated_ty_data(ty)
    }

    fn trait_datum(&self, id: TraitId<ChalkIr>) -> Arc<TraitDatum<ChalkIr>> {
        self.chalk_db.trait_datum(id)
    }

    fn impl_datum(&self, id: ImplId<ChalkIr>) -> Arc<ImplDatum<ChalkIr>> {
        self.chalk_db.impl_datum(id)
    }

    fn associated_ty_value(
        &self,
        id: AssociatedTyValueId<ChalkIr>,
    ) -> Arc<AssociatedTyValue<ChalkIr>> {
        self.chalk_db.associated_ty_value(id)
    }

    fn opaque_ty_data(&self, id: OpaqueTyId<ChalkIr>) -> Arc<OpaqueTyDatum<ChalkIr>> {
        self.chalk_db.opaque_ty_data(id)
    }

    fn struct_datum(&self, id: StructId<ChalkIr>) -> Arc<StructDatum<ChalkIr>> {
        self.chalk_db.struct_datum(id)
    }

    fn as_struct_id(&self, type_name: &TypeName<ChalkIr>) -> Option<StructId<ChalkIr>> {
        self.chalk_db.as_struct_id(type_name)
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<ChalkIr>,
        parameters: &[Parameter<ChalkIr>],
    ) -> Vec<ImplId<ChalkIr>> {
        self.chalk_db.impls_for_trait(trait_id, parameters)
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId<ChalkIr>) -> Vec<ImplId<ChalkIr>> {
        self.chalk_db.local_impls_to_coherence_check(trait_id)
    }

    fn impl_provided_for(
        &self,
        auto_trait_id: TraitId<ChalkIr>,
        struct_id: StructId<ChalkIr>,
    ) -> bool {
        self.chalk_db.impl_provided_for(auto_trait_id, struct_id)
    }

    fn well_known_trait_id(&self, well_known_trait: WellKnownTrait) -> Option<TraitId<ChalkIr>> {
        self.chalk_db.well_known_trait_id(well_known_trait)
    }

    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }
}

#[test]
fn unwind_safety() {
    use self::MockDatabase;
    use chalk_integration::lowering::LowerGoal;
    use chalk_integration::query::LoweringDatabase;
    use chalk_solve::ext::GoalExt;
    use std::panic;

    // lower program
    let mut db = lower_program_with_db! {
        program {
            struct Foo { }
            trait Bar { }
            impl Bar for Foo { }
        }
        database MockDatabase
    };

    let program = db.chalk_db.checked_program().unwrap();

    // lower goal
    let goal = lower_goal! {
        goal {
            Foo: Bar
        }
        program &*program
    };
    let peeled_goal = goal.into_peeled_goal(db.interner());

    // solve goal but this will panic
    db.panic = true;
    let result = panic::catch_unwind(|| {
        db.solve(&peeled_goal);
    });
    assert!(result.is_err() == true);

    // solve again but without panicking this time
    db.panic = false;
    assert!(db.solve(&peeled_goal).is_some());
}
