use cast::{Cast, Caster};
use fold::shift::Shift;
use ir::{self, ToParameter};

mod default;
mod wf;

impl ir::Program {
    pub fn environment(&self) -> ir::ProgramEnvironment {
        // Construct the set of *clauses*; these are sort of a compiled form
        // of the data above that always has the form:
        //
        //       forall P0...Pn. Something :- Conditions
        let mut program_clauses = vec![];

        program_clauses.extend(self.custom_clauses.iter().cloned());

        program_clauses.extend(
            self.struct_data
                .values()
                .flat_map(|d| d.to_program_clauses()),
        );
        program_clauses.extend(
            self.trait_data
                .values()
                .flat_map(|d| d.to_program_clauses()),
        );
        program_clauses.extend(
            self.associated_ty_data
                .values()
                .flat_map(|d| d.to_program_clauses(self)),
        );
        program_clauses.extend(self.default_impl_data.iter().map(|d| d.to_program_clause()));

        // Adds clause that defines the Derefs domain goal:
        // forall<T, U> { Derefs(T, U) :- ProjectionEq(<T as Deref>::Target = U>) }
        if let Some(trait_id) = self.lang_items.get(&ir::LangItem::DerefTrait) {
            // Find `Deref::Target`.
            let associated_ty_id = self.associated_ty_data.values()
                                                        .find(|d| d.trait_id == *trait_id)
                                                        .expect("Deref has no assoc item")
                                                        .id;
            let t = || ir::Ty::Var(0);
            let u = || ir::Ty::Var(1);
            program_clauses.push(ir::Binders {
                binders: vec![ir::ParameterKind::Ty(()), ir::ParameterKind::Ty(())],
                value: ir::ProgramClauseImplication {
                    consequence: ir::DomainGoal::Derefs(ir::Derefs { source: t(), target: u() }),
                    conditions: vec![ir::ProjectionEq {
                        projection: ir::ProjectionTy {
                            associated_ty_id,
                            parameters: vec![t().cast()]
                        },
                        ty: u(),
                    }.cast()]
                },
            }.cast());
        }

        for datum in self.impl_data.values() {
            // If we encounter a negative impl, do not generate any rule. Negative impls
            // are currently just there to deactivate default impls for auto traits.
            if datum.binders.value.trait_ref.is_positive() {
                program_clauses.push(datum.to_program_clause());
                program_clauses.extend(
                    datum
                        .binders
                        .value
                        .associated_ty_values
                        .iter()
                        .flat_map(|atv| atv.to_program_clauses(self, datum)),
                );
            }
        }

        let trait_data = self.trait_data.clone();
        let associated_ty_data = self.associated_ty_data.clone();

        ir::ProgramEnvironment {
            trait_data,
            associated_ty_data,
            program_clauses,
        }
    }
}

impl ir::ImplDatum {
    /// Given `impl<T: Clone> Clone for Vec<T>`, generate:
    ///
    /// ```notrust
    /// forall<T> { (Vec<T>: Clone) :- (T: Clone) }
    /// ```
    fn to_program_clause(&self) -> ir::ProgramClause {
        self.binders.map_ref(|bound| {
            ir::ProgramClauseImplication {
                consequence: bound.trait_ref.trait_ref().clone().cast(),
                conditions: bound
                    .where_clauses
                    .iter()
                    .cloned()
                    .casted()
                    .collect(),
            }
        }).cast()
    }
}

impl ir::DefaultImplDatum {
    /// For each accessible type `T` in a struct which needs a default implementation for the auto
    /// trait `Foo` (accessible types are the struct fields types), we add a bound `T: Foo` (which
    /// is then expanded with `WF(T: Foo)`). For example, given:
    ///
    /// ```notrust
    /// #[auto] trait Send { }
    ///
    /// struct MyList<T> {
    ///     data: T,
    ///     next: Box<Option<MyList<T>>>,
    /// }
    ///
    /// ```
    ///
    /// generate:
    ///
    /// ```notrust
    /// forall<T> {
    ///     (MyList<T>: Send) :-
    ///         (T: Send),
    ///         (Box<Option<MyList<T>>>: Send)
    /// }
    /// ```
    fn to_program_clause(&self) -> ir::ProgramClause {
        self.binders.map_ref(|bound| {
            ir::ProgramClauseImplication {
                consequence: bound.trait_ref.clone().cast(),
                conditions: {
                    let wc = bound.accessible_tys.iter().cloned().map(|ty| {
                        ir::TraitRef {
                            trait_id: bound.trait_ref.trait_id,
                            parameters: vec![ir::ParameterKind::Ty(ty)],
                        }
                    });

                    wc.casted().collect()
                },
            }
        }).cast()
    }
}

impl ir::AssociatedTyValue {
    /// Given:
    ///
    /// ```notrust
    /// impl<T> Iterable for Vec<T> {
    ///     type IntoIter<'a> where T: 'a = Iter<'a, T>;
    /// }
    /// ```
    ///
    /// generate:
    ///
    /// ```notrust
    /// forall<'a, T> {
    ///     Normalize(<Vec<T> as Iterable>::IntoIter<'a> -> Iter<'a, T>>) :-
    ///         (Vec<T>: Iterable),  // (1)
    ///         (T: 'a)              // (2)
    /// }
    /// ```
    ///
    /// and:
    ///
    /// ```notrust
    /// forall<'a, T> {
    ///     UnselectedNormalize(Vec<T>::IntoIter<'a> -> Iter<'a, T>) :-
    ///         InScope(Iterable),
    ///         Normalize(<Vec<T> as Iterable>::IntoIter<'a> -> Iter<'a, T>)
    /// }
    /// ```
    fn to_program_clauses(
        &self,
        program: &ir::Program,
        impl_datum: &ir::ImplDatum,
    ) -> Vec<ir::ProgramClause> {
        // Begin with the innermost parameters (`'a`) and then add those from impl (`T`).
        let all_binders: Vec<_> = self.value
            .binders
            .iter()
            .cloned()
            .chain(impl_datum.binders.binders.iter().cloned())
            .collect();

        // Assemble the full list of conditions for projection to be valid.
        // This comes in two parts, marked as (1) and (2) in example above:
        //
        // 1. require that the trait is implemented
        // 2. any where-clauses from the `type` declaration in the impl
        let impl_trait_ref = impl_datum
            .binders
            .value
            .trait_ref
            .trait_ref()
            .up_shift(self.value.len());
        let conditions: Vec<ir::Goal> = vec![impl_trait_ref.clone().cast()];

        // Bound parameters + `Self` type of the trait-ref
        let parameters: Vec<_> = {
            // First add refs to the bound parameters (`'a`, in above example)
            let parameters = self.value.binders.iter().zip(0..).map(|p| p.to_parameter());

            // Then add the `Self` type (`Vec<T>`, in above example)
            parameters
                .chain(Some(impl_trait_ref.parameters[0].clone()))
                .collect()
        };

        let projection = ir::ProjectionTy {
            associated_ty_id: self.associated_ty_id,

            // Add the remaining parameters of the trait-ref if any
            parameters: parameters.iter()
                                  .chain(&impl_trait_ref.parameters[1..])
                                  .cloned()
                                  .collect(),
        };

        let normalize_goal = ir::DomainGoal::Normalize(ir::Normalize {
            projection: projection.clone(),
            ty: self.value.value.ty.clone(),
        });

        // Determine the normalization
        let normalization = ir::Binders {
            binders: all_binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: normalize_goal.clone(),
                conditions: conditions.clone(),
            },
        }.cast();

        let unselected_projection = ir::UnselectedProjectionTy {
            type_name: program.associated_ty_data[&self.associated_ty_id]
                .name
                .clone(),
            parameters: parameters,
        };

        let unselected_normalization = ir::Binders {
            binders: all_binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::UnselectedNormalize(ir::UnselectedNormalize {
                    projection: unselected_projection,
                    ty: self.value.value.ty.clone(),
                }),
                conditions: vec![
                    normalize_goal.cast(),
                    ir::DomainGoal::InScope(impl_trait_ref.trait_id).cast(),
                ],
            },
        }.cast();

        vec![normalization, unselected_normalization]
    }
}

impl ir::StructDatum {
    fn to_program_clauses(&self) -> Vec<ir::ProgramClause> {
        // Given:
        //
        //    struct Foo<T: Eq> { }
        //
        // we generate the following clause:
        //
        //    forall<T> { WF(Foo<T>) :- (T: Eq). }
        //    forall<T> { FromEnv(T: Eq) :- FromEnv(Foo<T>). }
        //
        // If the type Foo is not marked `extern`, we also generate:
        //
        //    forall<T> { IsLocalTy(Foo<T>) }
        //
        // Given an `extern` type that is also fundamental:
        //
        //    #[fundamental]
        //    extern struct Box<T> {}
        //
        // We generate the following clause:
        //
        //    forall<T> { IsLocalTy(Box<T>) :- IsLocalTy(T) }

        let wf = self.binders.map_ref(|bound_datum| {
            ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::WellFormedTy(bound_datum.self_ty.clone().cast()),

                conditions: {
                    bound_datum.where_clauses
                                .iter()
                                .cloned()
                                .casted()
                                .collect()
                },
            }
        }).cast();

        let mut clauses = vec![wf];

        // Types that are not marked `extern` satisfy IsLocal(TypeName)
        if !self.binders.value.flags.external {
            // `IsLocalTy(Ty)` depends *only* on whether the type is marked extern and nothing else
            let is_local = self.binders.map_ref(|bound_datum| ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::IsLocalTy(bound_datum.self_ty.clone().cast()),
                conditions: Vec::new(),
            }).cast();

            clauses.push(is_local);
        } else if self.binders.value.flags.fundamental {
            // If a type is `extern`, but is also `#[fundamental]`, it satisfies IsLocalTy
            // if and only if its parameters satisfy IsLocalTy

            // Fundamental types must always have at least one type parameter for this rule to
            // make any sense. We currently do not have have any fundamental types with more than
            // one type parameter, nor do we know what the behaviour for that should be. Thus, we
            // are asserting here that there is only a single type parameter until the day when
            // someone makes a decision about how that should behave.
            assert_eq!(self.binders.value.self_ty.len_type_parameters(), 1,
                "Only fundamental types with a single parameter are supported");

            let local_fundamental = self.binders.map_ref(|bound_datum| ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::IsLocalTy(bound_datum.self_ty.clone().cast()),
                conditions: vec![
                    ir::DomainGoal::IsLocalTy(
                        // This unwrap is safe because we asserted above for the presence of a type
                        // parameter
                        bound_datum.self_ty.first_type_parameter().unwrap()
                    ).cast(),
                ],
            }).cast();

            clauses.push(local_fundamental);
        }

        let condition = ir::DomainGoal::FromEnvTy(self.binders.value.self_ty.clone().cast());

        for wc in self.binders
                      .value
                      .where_clauses
                      .iter()
                      .cloned()
                      .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
        {
            // We move the binders of the where-clause to the left, e.g. if we had:
            //
            // `forall<T> { WellFormed(Foo<T>) :- forall<'a> Implemented(T: Fn(&'a i32)) }`
            //
            // then the reverse rule will be:
            //
            // `forall<'a, T> { FromEnv(T: Fn(&'a i32)) :- FromEnv(Foo<T>) }`
            //
            let shift = wc.binders.len();
            clauses.push(ir::Binders {
                binders: wc.binders.into_iter().chain(self.binders.binders.clone()).collect(),
                value: ir::ProgramClauseImplication {
                    consequence: wc.value,
                    conditions: vec![condition.clone().up_shift(shift).cast()],
                }
            }.cast());
        }

        clauses
    }
}

impl ir::TraitDatum {
    fn to_program_clauses(&self) -> Vec<ir::ProgramClause> {
        // Given:
        //
        //    trait Ord<T> where Self: Eq<T> { ... }
        //
        // we generate the following clause:
        //
        //    forall<Self, T> {
        //        WF(Self: Ord<T>) :- (Self: Ord<T>), WF(Self: Eq<T>)
        //    }
        //
        // and the reverse rules:
        //
        //    forall<Self, T> { (Self: Ord<T>) :- FromEnv(Self: Ord<T>) }
        //    forall<Self, T> { FromEnv(Self: Eq<T>) :- FromEnv(Self: Ord<T>) }

        let trait_ref_impl = ir::WhereClauseAtom::Implemented(
           self.binders.value.trait_ref.clone()
        );

        let wf = self.binders.map_ref(|bound| {
            ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::WellFormed(trait_ref_impl.clone()),

                conditions: {
                    bound.where_clauses
                            .iter()
                            .cloned()
                            .map(|wc| wc.map(|bound| bound.into_well_formed_goal()).cast())
                            .chain(Some(ir::DomainGoal::Holds(trait_ref_impl.clone()).cast()))
                            .collect()
                },
            }
        }).cast();

        let mut clauses = vec![wf];
        let condition = ir::DomainGoal::FromEnv(trait_ref_impl.clone());

        for wc in self.binders
                      .value
                      .where_clauses
                      .iter()
                      .cloned()
                      .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
        {
            // We move the binders of the where-clause to the left for the reverse rules,
            // cf `StructDatum::to_program_clauses`.
            let shift = wc.binders.len();
            clauses.push(ir::Binders {
                binders: wc.binders.into_iter().chain(self.binders.binders.clone()).collect(),
                value: ir::ProgramClauseImplication {
                    consequence: wc.value,
                    conditions: vec![condition.clone().up_shift(shift).cast()],
                }
            }.cast());
        }

        clauses.push(self.binders.map_ref(|_| {
            ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::Holds(trait_ref_impl),
                conditions: vec![condition.cast()],
            }
        }).cast());

        clauses
    }
}

impl ir::AssociatedTyDatum {
    fn to_program_clauses(&self, program: &ir::Program) -> Vec<ir::ProgramClause> {
        // For each associated type, we define the "projection
        // equality" rules. There are always two; one for a successful normalization,
        // and one for the "fallback" notion of equality.
        //
        // Given:
        //
        //    trait Foo {
        //        type Assoc;
        //    }
        //
        // we generate the 'fallback' rule:
        //
        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = (Foo::Assoc)<T>) :-
        //            T: Foo
        //    }
        //
        // and
        //
        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :-
        //            Normalize(<T as Foo>::Assoc -> U)
        //    }
        //
        // We used to generate an "elaboration" rule like this:
        //
        //    forall<T> {
        //        T: Foo :-
        //            exists<U> { ProjectionEq(<T as Foo>::Assoc = U) }
        //    }
        //
        // but this caused problems with the recursive solver. In
        // particular, whenever normalization is possible, we cannot
        // solve that projection uniquely, since we can now elaborate
        // `ProjectionEq` to fallback *or* normalize it. So instead we
        // handle this kind of reasoning through the `FromEnv` predicate.
        //
        // We also generate rules specific to WF requirements and implied bounds,
        // see below.

        let binders: Vec<_> = self.parameter_kinds
            .iter()
            .map(|pk| pk.map(|_| ()))
            .collect();
        let parameters: Vec<_> = binders.iter().zip(0..).map(|p| p.to_parameter()).collect();
        let projection = ir::ProjectionTy {
            associated_ty_id: self.id,
            parameters: parameters.clone(),
        };

        // Retrieve the trait ref embedding the associated type
        let trait_ref = {
            let (associated_ty_data, trait_params, _) = program.split_projection(&projection);
            ir::TraitRef {
                trait_id: associated_ty_data.trait_id,
                parameters: trait_params.to_owned(),
            }
        };

        // Construct an application from the projection. So if we have `<T as Iterator>::Item`,
        // we would produce `(Iterator::Item)<T>`.
        let app = ir::ApplicationTy {
            name: ir::TypeName::AssociatedType(self.id),
            parameters,
        };
        let app_ty = ir::Ty::Apply(app);

        let mut clauses = vec![];

        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = (Foo::Assoc)<T>) :-
        //            T: Foo
        //    }
        clauses.push(ir::Binders {
            binders: binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: ir::ProjectionEq {
                    projection: projection.clone(),
                    ty: app_ty.clone(),
                }.cast(),
                conditions: vec![trait_ref.clone().cast()],
            },
        }.cast());

        // The above application type is always well-formed, and `<T as Foo>::Assoc` will
        // unify with `(Foo::Assoc)<T>` only if `T: Foo`, because of the above rule, so we have:
        //
        //    forall<T> {
        //        WellFormed((Foo::Assoc)<T>).
        //    }
        clauses.push(ir::Binders {
            binders: binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::WellFormedTy(app_ty).cast(),
                conditions: vec![],
            },
        }.cast());

        // add new type parameter U
        let mut binders = binders;
        binders.push(ir::ParameterKind::Ty(()));
        let ty = ir::Ty::Var(binders.len() - 1);

        // `Normalize(<T as Foo>::Assoc -> U)`
        let normalize = ir::Normalize { projection: projection.clone(), ty: ty.clone() };

        // `ProjectionEq(<T as Foo>::Assoc = U)`
        let projection_eq = ir::ProjectionEq { projection: projection.clone(), ty };

        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :-
        //            Normalize(<T as Foo>::Assoc -> U)
        //    }
        clauses.push(ir::Binders {
            binders: binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: projection_eq.clone().cast(),
                conditions: vec![normalize.clone().cast()],
            },
        }.cast());


        let projection_wc = ir::WhereClauseAtom::ProjectionEq(projection_eq.clone());
        let trait_ref_wc = ir::WhereClauseAtom::Implemented(trait_ref.clone());

        // We generate a proxy rule for the well-formedness of `T: Foo<Assoc = U>` which really
        // means two things: `T: Foo` and `Normalize(<T as Foo>::Assoc -> U)`. So we have the
        // following rule:
        //
        //    forall<T> {
        //        WellFormed(T: Foo<Assoc = U>) :-
        //            WellFormed(T: Foo), ProjectionEq(<T as Foo>::Assoc = U)
        //    }
        clauses.push(ir::Binders {
            binders: binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::WellFormed(projection_wc.clone()),
                conditions: vec![
                    ir::DomainGoal::WellFormed(trait_ref_wc.clone()).cast(),
                    projection_eq.clone().cast()
                ],
            }
        }.cast());

        // We also have two proxy reverse rules, the first one being:
        //
        //    forall<T> {
        //        FromEnv(T: Foo) :- FromEnv(T: Foo<Assoc = U>)
        //    }
        clauses.push(ir::Binders {
            binders: binders.clone(),
            value: ir::ProgramClauseImplication {
                consequence: ir::DomainGoal::FromEnv(trait_ref_wc).cast(),
                conditions: vec![ir::DomainGoal::FromEnv(projection_wc.clone()).cast()],
            },
        }.cast());

        // And the other one being:
        //
        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :- FromEnv(T: Foo<Assoc = U>)
        //    }
        clauses.push(ir::Binders {
            binders,
            value: ir::ProgramClauseImplication {
                consequence: projection_eq.clone().cast(),
                conditions: vec![ir::DomainGoal::FromEnv(projection_wc).cast()],
            },
        }.cast());

        clauses
    }
}
