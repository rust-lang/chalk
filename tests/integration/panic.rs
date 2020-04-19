use chalk_ir::interner::ChalkIr;
use chalk_ir::AssocTypeId;
use chalk_ir::ImplId;
use chalk_ir::OpaqueTyId;
use chalk_ir::Parameter;
use chalk_ir::ProgramClause;
use chalk_ir::StructId;
use chalk_ir::TraitId;
use chalk_ir::TypeName;
use chalk_rust_ir::AssociatedTyDatum;
use chalk_rust_ir::AssociatedTyValue;
use chalk_rust_ir::AssociatedTyValueId;
use chalk_rust_ir::ImplDatum;
use chalk_rust_ir::OpaqueTyDatum;
use chalk_rust_ir::StructDatum;
use chalk_rust_ir::TraitDatum;
use chalk_rust_ir::WellKnownTrait;
use chalk_solve::RustIrDatabase;
use chalk_solve::SolverChoice;
use std::sync::Arc;

#[derive(Debug, Default)]
struct MockDatabase {
    panic: bool,
}

#[allow(unused_variables)]
impl RustIrDatabase<ChalkIr> for MockDatabase {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        if self.panic {
            panic!("test panic");
        } else {
            vec![]
        }
    }

    fn associated_ty_data(&self, ty: AssocTypeId<ChalkIr>) -> Arc<AssociatedTyDatum<ChalkIr>> {
        unimplemented!()
    }

    fn trait_datum(&self, id: TraitId<ChalkIr>) -> Arc<TraitDatum<ChalkIr>> {
        assert_eq!(id.0.index, 0);
        return Arc::new(chalk_rust_ir::TraitDatum {
            id,
            binders: chalk_ir::Binders::new(
                chalk_ir::ParameterKinds::new(&ChalkIr),
                chalk_rust_ir::TraitDatumBound {
                    where_clauses: vec![],
                },
            ),
            flags: chalk_rust_ir::TraitFlags {
                auto: false,
                marker: false,
                upstream: false,
                fundamental: false,
                non_enumerable: false,
                coinductive: false,
            },
            associated_ty_ids: vec![],
            well_known: None,
        });
    }

    fn impl_datum(&self, id: ImplId<ChalkIr>) -> Arc<ImplDatum<ChalkIr>> {
        unimplemented!()
    }

    fn associated_ty_value(
        &self,
        id: AssociatedTyValueId<ChalkIr>,
    ) -> Arc<AssociatedTyValue<ChalkIr>> {
        unimplemented!()
    }

    fn opaque_ty_data(&self, id: OpaqueTyId<ChalkIr>) -> Arc<OpaqueTyDatum<ChalkIr>> {
        unimplemented!()
    }

    fn struct_datum(&self, id: StructId<ChalkIr>) -> Arc<StructDatum<ChalkIr>> {
        unimplemented!()
    }

    fn as_struct_id(&self, type_name: &TypeName<ChalkIr>) -> Option<StructId<ChalkIr>> {
        unimplemented!()
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<ChalkIr>,
        parameters: &[Parameter<ChalkIr>],
    ) -> Vec<ImplId<ChalkIr>> {
        unimplemented!()
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId<ChalkIr>) -> Vec<ImplId<ChalkIr>> {
        unimplemented!()
    }

    fn impl_provided_for(
        &self,
        auto_trait_id: TraitId<ChalkIr>,
        struct_id: StructId<ChalkIr>,
    ) -> bool {
        unimplemented!()
    }

    fn well_known_trait_id(&self, well_known_trait: WellKnownTrait) -> Option<TraitId<ChalkIr>> {
        unimplemented!()
    }

    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }
}

#[test]
fn unwind_safety() {
    use self::MockDatabase;
    use chalk_ir::*;
    use std::panic;

    // lower program
    /*
    let mut db = lower_program_with_db! {
        program {
            struct Foo { }
            trait Bar { }
            impl Bar for Foo { }
        }
        database MockDatabase
    };

    let program = db.chalk_db.checked_program().unwrap();
    */
    let mut db = MockDatabase { panic: false };

    let peeled_goal: UCanonical<InEnvironment<Goal<ChalkIr>>> = UCanonical {
        canonical: Canonical {
            binders: CanonicalVarKinds::new(&ChalkIr),
            value: InEnvironment {
                environment: Environment::new(&ChalkIr),
                goal: GoalData::DomainGoal(DomainGoal::Holds(WhereClause::Implemented(TraitRef {
                    trait_id: TraitId(interner::RawId { index: 0 }),
                    substitution: Substitution::from1(
                        &ChalkIr,
                        ParameterKind::Ty(
                            TyData::Apply(ApplicationTy {
                                name: TypeName::Struct(StructId(interner::RawId { index: 1 })),
                                substitution: Substitution::empty(&ChalkIr),
                            })
                            .intern(&ChalkIr),
                        )
                        .intern(&ChalkIr),
                    ),
                })))
                .intern(&ChalkIr),
            },
        },
        universes: 1,
    };

    let mut solver = SolverChoice::slg_default().into_solver();
    // solve goal but this will panic
    db.panic = true;
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        solver.solve(&db, &peeled_goal);
    }));
    assert!(result.is_err() == true);

    // solve again but without panicking this time
    db.panic = false;
    assert!(solver.solve(&db, &peeled_goal).is_some());
}
