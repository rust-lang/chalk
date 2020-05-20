use chalk_integration::interner::{ChalkIr, RawId};
use chalk_ir::*;
use chalk_rust_ir::*;
use chalk_solve::{RustIrDatabase, SolverChoice};
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
        Arc::new(chalk_rust_ir::TraitDatum {
            id,
            binders: Binders::new(
                VariableKinds::new(&ChalkIr),
                TraitDatumBound {
                    where_clauses: vec![],
                },
            ),
            flags: TraitFlags {
                auto: false,
                marker: false,
                upstream: false,
                fundamental: false,
                non_enumerable: false,
                coinductive: false,
            },
            associated_ty_ids: vec![],
            well_known: None,
        })
    }

    fn impl_datum(&self, id: ImplId<ChalkIr>) -> Arc<ImplDatum<ChalkIr>> {
        assert_eq!(id.0.index, 1);

        let substitution = Ty::new(
            &ChalkIr,
            ApplicationTy {
                name: TypeName::Adt(AdtId(RawId { index: 1 })),
                substitution: Substitution::empty(&ChalkIr),
            },
        );

        let binders = Binders::new(
            VariableKinds::new(&ChalkIr),
            ImplDatumBound {
                trait_ref: TraitRef {
                    trait_id: TraitId(RawId { index: 0 }),
                    substitution: Substitution::from1(&ChalkIr, substitution),
                },
                where_clauses: vec![],
            },
        );

        Arc::new(ImplDatum {
            polarity: Polarity::Positive,
            binders,
            impl_type: ImplType::Local,
            associated_ty_value_ids: vec![],
        })
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

    fn adt_datum(&self, id: AdtId<ChalkIr>) -> Arc<AdtDatum<ChalkIr>> {
        unimplemented!()
    }

    fn fn_def_datum(&self, fn_def_id: FnDefId<ChalkIr>) -> Arc<FnDefDatum<ChalkIr>> {
        unimplemented!()
    }

    fn impls_for_trait(
        &self,
        trait_id: TraitId<ChalkIr>,
        parameters: &[GenericArg<ChalkIr>],
    ) -> Vec<ImplId<ChalkIr>> {
        assert_eq!(trait_id.0.index, 0);
        vec![ImplId(RawId { index: 1 })]
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId<ChalkIr>) -> Vec<ImplId<ChalkIr>> {
        unimplemented!()
    }

    fn impl_provided_for(
        &self,
        auto_trait_id: TraitId<ChalkIr>,
        struct_id: AdtId<ChalkIr>,
    ) -> bool {
        unimplemented!()
    }

    fn well_known_trait_id(&self, well_known_trait: WellKnownTrait) -> Option<TraitId<ChalkIr>> {
        unimplemented!()
    }

    fn program_clauses_for_env(
        &self,
        environment: &Environment<ChalkIr>,
    ) -> ProgramClauses<ChalkIr> {
        ProgramClauses::new(&ChalkIr)
    }

    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }

    fn is_object_safe(&self, trait_id: TraitId<ChalkIr>) -> bool {
        unimplemented!()
    }
}

#[test]
fn unwind_safety() {
    use self::MockDatabase;
    use chalk_integration::interner::{self, ChalkIr};
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
                        Ty::new(
                            &ChalkIr,
                            ApplicationTy {
                                name: TypeName::Adt(AdtId(interner::RawId { index: 1 })),
                                substitution: Substitution::empty(&ChalkIr),
                            },
                        ),
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
    assert!(result.is_err());

    // solve again but without panicking this time
    db.panic = false;
    assert!(solver.solve(&db, &peeled_goal).is_some());
}
