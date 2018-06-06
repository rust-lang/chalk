use cast::{Cast, Caster};
use fold::shift::Shift;
use fold::Subst;
use ir::*;
use std::iter;

mod default;
mod wf;

impl Program {
    pub fn environment(&self) -> ProgramEnvironment {
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
        if let Some(trait_id) = self.lang_items.get(&LangItem::DerefTrait) {
            // Find `Deref::Target`.
            let associated_ty_id = self.associated_ty_data.values()
                                                        .find(|d| d.trait_id == *trait_id)
                                                        .expect("Deref has no assoc item")
                                                        .id;
            let t = || Ty::Var(0);
            let u = || Ty::Var(1);
            program_clauses.push(Binders {
                binders: vec![ParameterKind::Ty(()), ParameterKind::Ty(())],
                value: ProgramClauseImplication {
                    consequence: DomainGoal::Derefs(Derefs { source: t(), target: u() }),
                    conditions: vec![ProjectionEq {
                        projection: ProjectionTy {
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

        ProgramEnvironment {
            trait_data,
            associated_ty_data,
            program_clauses,
        }
    }
}

impl ImplDatum {
    /// Given `impl<T: Clone> Clone for Vec<T>`, generate:
    ///
    /// ```notrust
    /// forall<T> { (Vec<T>: Clone) :- (T: Clone) }
    /// ```
    fn to_program_clause(&self) -> ProgramClause {
        self.binders.map_ref(|bound| {
            ProgramClauseImplication {
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

impl DefaultImplDatum {
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
    fn to_program_clause(&self) -> ProgramClause {
        self.binders.map_ref(|bound| {
            ProgramClauseImplication {
                consequence: bound.trait_ref.clone().cast(),
                conditions: {
                    let wc = bound.accessible_tys.iter().cloned().map(|ty| {
                        TraitRef {
                            trait_id: bound.trait_ref.trait_id,
                            parameters: vec![ParameterKind::Ty(ty)],
                        }
                    });

                    wc.casted().collect()
                },
            }
        }).cast()
    }
}

impl AssociatedTyValue {
    /// Given the following trait:
    ///
    /// ```notrust
    /// trait Iterable {
    ///     type IntoIter<'a>: 'a;
    /// }
    /// ```
    ///
    /// Then for the following impl:
    /// ```notrust
    /// impl<T> Iterable for Vec<T> {
    ///     type IntoIter<'a> = Iter<'a, T>;
    /// }
    /// ```
    ///
    /// we generate:
    ///
    /// ```notrust
    /// forall<'a, T> {
    ///     Normalize(<Vec<T> as Iterable>::IntoIter<'a> -> Iter<'a, T>>) :-
    ///         (Vec<T>: Iterable),  // (1)
    ///         (Iter<'a, T>: 'a)    // (2)
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
        program: &Program,
        impl_datum: &ImplDatum,
    ) -> Vec<ProgramClause> {
        let associated_ty = &program.associated_ty_data[&self.associated_ty_id];

        // Begin with the innermost parameters (`'a`) and then add those from impl (`T`).
        let all_binders: Vec<_> = self.value
            .binders
            .iter()
            .chain(impl_datum.binders.binders.iter())
            .cloned()
            .collect();

        let impl_trait_ref = impl_datum.binders
                                       .value
                                       .trait_ref
                                       .trait_ref()
                                       .up_shift(self.value.len());

        let all_parameters: Vec<_> =
            self.value.binders
                      .iter()
                      .zip(0..)
                      .map(|p| p.to_parameter())
                      .chain(impl_trait_ref.parameters.iter().cloned())
                      .collect();

        // Assemble the full list of conditions for projection to be valid.
        // This comes in two parts, marked as (1) and (2) in example above:
        //
        // 1. require that the trait is implemented
        // 2. any where-clauses from the `type` declaration in the trait: the
        //    parameters must be substituted with those of the impl
        let where_clauses =
            associated_ty.where_clauses
                         .iter()
                         .map(|wc| Subst::apply(&all_parameters, wc))
                         .casted();

        let conditions: Vec<Goal> =
            where_clauses
            .chain(Some(impl_trait_ref.clone().cast()))
            .collect();

        // Bound parameters + `Self` type of the trait-ref
        let parameters: Vec<_> = {
            // First add refs to the bound parameters (`'a`, in above example)
            let parameters = self.value.binders.iter().zip(0..).map(|p| p.to_parameter());

            // Then add the `Self` type (`Vec<T>`, in above example)
            parameters
                .chain(Some(impl_trait_ref.parameters[0].clone()))
                .collect()
        };

        let projection = ProjectionTy {
            associated_ty_id: self.associated_ty_id,

            // Add the remaining parameters of the trait-ref, if any
            parameters: parameters.iter()
                                  .chain(&impl_trait_ref.parameters[1..])
                                  .cloned()
                                  .collect(),
        };

        let normalize_goal = DomainGoal::Normalize(Normalize {
            projection: projection.clone(),
            ty: self.value.value.ty.clone(),
        });

        // Determine the normalization
        let normalization = Binders {
            binders: all_binders.clone(),
            value: ProgramClauseImplication {
                consequence: normalize_goal.clone(),
                conditions: conditions,
            },
        }.cast();

        let unselected_projection = UnselectedProjectionTy {
            type_name: associated_ty.name.clone(),
            parameters: parameters,
        };

        let unselected_normalization = Binders {
            binders: all_binders.clone(),
            value: ProgramClauseImplication {
                consequence: DomainGoal::UnselectedNormalize(UnselectedNormalize {
                    projection: unselected_projection,
                    ty: self.value.value.ty.clone(),
                }),
                conditions: vec![
                    normalize_goal.cast(),
                    DomainGoal::InScope(impl_trait_ref.trait_id).cast(),
                ],
            },
        }.cast();

        vec![normalization, unselected_normalization]
    }
}

impl StructDatum {
    fn to_program_clauses(&self) -> Vec<ProgramClause> {
        // Given:
        //
        //    struct Foo<T: Eq> { }
        //
        // we generate the following clause:
        //
        //    forall<T> { WF(Foo<T>) :- (T: Eq). }
        //    forall<T> { FromEnv(T: Eq) :- FromEnv(Foo<T>). }
        //
        // If the type Foo is marked `extern`, we also generate:
        //
        //    forall<T> { IsExternal(Foo<T>) }
        //    forall<T> { IsDeeplyExternal(Foo<T>) :- IsDeeplyExternal(T) }
        //
        // Otherwise, if the type Foo is not marked `extern`, we generate:
        //
        //    forall<T> { IsLocal(Foo<T>) }
        //
        // Given an `extern` type that is also fundamental:
        //
        //    #[fundamental]
        //    extern struct Box<T> {}
        //
        // We generate the following clause:
        //
        //    forall<T> { IsLocal(Box<T>) :- IsLocal(T) }
        //    forall<T> { IsExternal(Box<T>) :- IsExternal(T) }
        //    forall<T> { IsDeeplyExternal(Box<T>) :- IsDeeplyExternal(T) }

        let wf = self.binders.map_ref(|bound_datum| {
            ProgramClauseImplication {
                consequence: WellFormed::Ty(bound_datum.self_ty.clone().cast()).cast(),

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
            let is_local = self.binders.map_ref(|bound_datum| ProgramClauseImplication {
                consequence: DomainGoal::IsLocal(bound_datum.self_ty.clone().cast()),
                conditions: Vec::new(),
            }).cast();

            clauses.push(is_local);
        } else if self.binders.value.flags.fundamental {
            // If a type is `extern`, but is also `#[fundamental]`, it satisfies IsLocal
            // if and only if its parameters satisfy IsLocal

            // Fundamental types must always have at least one type parameter for this rule to
            // make any sense. We currently do not have have any fundamental types with more than
            // one type parameter, nor do we know what the behaviour for that should be. Thus, we
            // are asserting here that there is only a single type parameter until the day when
            // someone makes a decision about how that should behave.
            assert_eq!(self.binders.value.self_ty.len_type_parameters(), 1,
                "Only fundamental types with a single parameter are supported");

            // Fundamental types often have rules in the form of:
            //     Goal(FundamentalType<T>) :- Goal(T)
            // This macro makes creating that kind of clause easy
            macro_rules! fundamental_rule {
                ($goal:ident) => {
                    clauses.push(self.binders.map_ref(|bound_datum| ProgramClauseImplication {
                        consequence: DomainGoal::$goal(bound_datum.self_ty.clone().cast()),
                        conditions: vec![
                        DomainGoal::$goal(
                            // This unwrap is safe because we asserted above for the presence of a type
                            // parameter
                            bound_datum.self_ty.first_type_parameter().unwrap()
                        ).cast(),
                        ],
                    }).cast());
                };
            }

            fundamental_rule!(IsLocal);
            fundamental_rule!(IsExternal);
            fundamental_rule!(IsDeeplyExternal);
        } else {
            // The type is just extern and not fundamental

            let is_external = self.binders.map_ref(|bound_datum| ProgramClauseImplication {
                consequence: DomainGoal::IsExternal(bound_datum.self_ty.clone().cast()),
                conditions: Vec::new(),
            }).cast();

            clauses.push(is_external);

            let is_deeply_external = self.binders.map_ref(|bound_datum| ProgramClauseImplication {
                consequence: DomainGoal::IsDeeplyExternal(bound_datum.self_ty.clone().cast()),
                conditions: bound_datum.self_ty.type_parameters()
                    .map(|ty| DomainGoal::IsDeeplyExternal(ty).cast())
                    .collect(),
            }).cast();

            clauses.push(is_deeply_external);
        }

        let condition = DomainGoal::FromEnv(
            FromEnv::Ty(self.binders.value.self_ty.clone().cast())
        );

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
            clauses.push(Binders {
                binders: wc.binders.into_iter().chain(self.binders.binders.clone()).collect(),
                value: ProgramClauseImplication {
                    consequence: wc.value,
                    conditions: vec![condition.clone().up_shift(shift).cast()],
                }
            }.cast());
        }

        clauses
    }
}

impl TraitDatum {
    fn to_program_clauses(&self) -> Vec<ProgramClause> {
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
        //
        // As specified in the orphan rules, if a trait is not marked `extern`, the current crate
        // can implement it for any type. To represent that, we generate:
        //
        //    // `Ord<T>` would not be `extern` when compiling `std`
        //    forall<Self, T> { LocalImplAllowed(Self: Ord<T>) }
        //
        // For traits that are `extern` (i.e. not in the current crate), the orphan rules dictate
        // that impls are allowed as long as at least one type parameter is local and each type
        // prior to that is *deeply* external. That means that each type prior to the first local
        // type cannot contain any of the type parameters of the impl.
        //
        // This rule is fairly complex, so we expand it and generate a program clause for each
        // possible case. This is represented as follows:
        //
        //    // for `extern trait Foo<T, U, V> where Self: Eq<T> { ... }`
        //    forall<Self, T, U, V> {
        //      LocalImplAllowed(Self: Foo<T, U, V>) :- IsLocal(Self)
        //    }
        //    forall<Self, T, U, V> {
        //      LocalImplAllowed(Self: Foo<T, U, V>) :-
        //          IsDeeplyExternal(Self),
        //          IsLocal(T)
        //    }
        //    forall<Self, T, U, V> {
        //      LocalImplAllowed(Self: Foo<T, U, V>) :-
        //          IsDeeplyExternal(Self),
        //          IsDeeplyExternal(T),
        //          IsLocal(U)
        //    }
        //    forall<Self, T, U, V> {
        //      LocalImplAllowed(Self: Foo<T, U, V>) :-
        //          IsDeeplyExternal(Self),
        //          IsDeeplyExternal(T),
        //          IsDeeplyExternal(U),
        //          IsLocal(V)
        //    }

        let trait_ref = self.binders.value.trait_ref.clone();

        let trait_ref_impl = WhereClause::Implemented(
           self.binders.value.trait_ref.clone()
        );

        let wf = self.binders.map_ref(|bound| {
            ProgramClauseImplication {
                consequence: WellFormed::Trait(trait_ref.clone()).cast(),

                conditions: {
                    bound.where_clauses
                            .iter()
                            .cloned()
                            .map(|wc| wc.map(|bound| bound.into_well_formed_goal()))
                            .casted()
                            .chain(Some(DomainGoal::Holds(trait_ref_impl.clone()).cast()))
                            .collect()
                },
            }
        }).cast();

        let mut clauses = vec![wf];

        if !self.binders.value.flags.external {
            let impl_allowed = self.binders.map_ref(|bound_datum|
                ProgramClauseImplication {
                    consequence: DomainGoal::LocalImplAllowed(bound_datum.trait_ref.clone()),
                    conditions: Vec::new(),
                }
            ).cast();

            clauses.push(impl_allowed);
        } else {
            // The number of parameters will always be at least 1 because of the Self parameter
            // that is automatically added to every trait. This is important because otherwise
            // the added program clauses would not have any conditions.

            let type_parameters: Vec<_> = self.binders.value.trait_ref.type_parameters().collect();

            for i in 0..type_parameters.len() {
                let impl_maybe_allowed = self.binders.map_ref(|bound_datum|
                    ProgramClauseImplication {
                        consequence: DomainGoal::LocalImplAllowed(bound_datum.trait_ref.clone()),
                        conditions: (0..i)
                            .map(|j| DomainGoal::IsDeeplyExternal(type_parameters[j].clone()).cast())
                            .chain(iter::once(DomainGoal::IsLocal(type_parameters[i].clone()).cast()))
                            .collect(),
                    }
                ).cast();

                clauses.push(impl_maybe_allowed);
            }
        }

        let condition = DomainGoal::FromEnv(FromEnv::Trait(trait_ref.clone()));

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
            clauses.push(Binders {
                binders: wc.binders.into_iter().chain(self.binders.binders.clone()).collect(),
                value: ProgramClauseImplication {
                    consequence: wc.value,
                    conditions: vec![condition.clone().up_shift(shift).cast()],
                }
            }.cast());
        }

        clauses.push(self.binders.map_ref(|_| {
            ProgramClauseImplication {
                consequence: DomainGoal::Holds(trait_ref_impl),
                conditions: vec![condition.cast()],
            }
        }).cast());

        clauses
    }
}

impl AssociatedTyDatum {
    fn to_program_clauses(&self, program: &Program) -> Vec<ProgramClause> {
        // For each associated type, we define the "projection
        // equality" rules. There are always two; one for a successful normalization,
        // and one for the "fallback" notion of equality.
        //
        // Given: (here, `'a` and `T` represent zero or more parameters)
        //
        //    trait Foo {
        //        type Assoc<'a, T>: Bounds where WC;
        //    }
        //
        // we generate the 'fallback' rule:
        //
        //    forall<Self, 'a, T> {
        //        ProjectionEq(<Self as Foo>::Assoc<'a, T> = (Foo::Assoc<'a, T>)<Self>).
        //    }
        //
        // and
        //
        //    forall<Self, 'a, T, U> {
        //        ProjectionEq(<T as Foo>::Assoc<'a, T> = U) :-
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
        let projection = ProjectionTy {
            associated_ty_id: self.id,
            parameters: parameters.clone(),
        };

        // Retrieve the trait ref embedding the associated type
        let trait_ref = {
            let (associated_ty_data, trait_params, _) = program.split_projection(&projection);
            TraitRef {
                trait_id: associated_ty_data.trait_id,
                parameters: trait_params.to_owned(),
            }
        };

        // Construct an application from the projection. So if we have `<T as Iterator>::Item`,
        // we would produce `(Iterator::Item)<T>`.
        let app = ApplicationTy {
            name: TypeName::AssociatedType(self.id),
            parameters,
        };
        let app_ty = Ty::Apply(app);

        let projection_eq = ProjectionEq {
            projection: projection.clone(),
            ty: app_ty.clone(),
        };

        let mut clauses = vec![];

        // Fallback rule. The solver uses this to move between the projection
        // and skolemized type.
        //
        //    forall<Self> {
        //        ProjectionEq(<Self as Foo>::Assoc = (Foo::Assoc)<Self>).
        //    }
        clauses.push(Binders {
            binders: binders.clone(),
            value: ProgramClauseImplication {
                consequence: projection_eq.clone().cast(),
                conditions: vec![],
            },
        }.cast());

        // Well-formedness of projection type.
        //
        //    forall<Self> {
        //        WellFormed((Foo::Assoc)<Self>) :- Self: Foo, WC.
        //    }
        clauses.push(Binders {
            binders: binders.clone(),
            value: ProgramClauseImplication {
                consequence: WellFormed::Ty(app_ty.clone()).cast(),
                conditions: iter::once(trait_ref.clone().cast())
                                .chain(self.where_clauses.iter().cloned().casted())
                                .collect(),
            },
        }.cast());

        // Assuming well-formedness of projection type means we can assume
        // the trait ref as well. Mostly used in function bodies.
        //
        //    forall<Self> {
        //        FromEnv(Self: Foo) :- FromEnv((Foo::Assoc)<Self>).
        //    }
        clauses.push(Binders {
            binders: binders.clone(),
            value: ProgramClauseImplication {
                consequence: FromEnv::Trait(trait_ref.clone()).cast(),
                conditions: vec![FromEnv::Ty(app_ty.clone()).cast()],
            }
        }.cast());

        // Reverse rule for where clauses.
        //
        //    forall<Self> {
        //        FromEnv(WC) :- FromEnv((Foo::Assoc)<Self>).
        //    }
        //
        // This is really a family of clauses, one for each where clause.
        clauses.extend(self.where_clauses.iter().map(|wc| {
            // Don't forget to move the binders to the left in case of higher-ranked where clauses.
            let shift = wc.binders.len();
            Binders {
                binders: wc.binders.iter().chain(binders.iter()).cloned().collect(),
                value: ProgramClauseImplication {
                    consequence: wc.value.clone().into_from_env_goal(),
                    conditions: vec![
                        FromEnv::Ty(app_ty.clone()).up_shift(shift).cast()
                    ],
                }
            }.cast()
        }));

        // Reverse rule for implied bounds.
        //
        //    forall<T> {
        //        FromEnv(<T as Foo>::Assoc: Bounds) :- FromEnv(Self: Foo)
        //    }
        clauses.extend(self.bounds_on_self().into_iter().map(|bound| {
            Binders {
                binders: binders.clone(),
                value: ProgramClauseImplication {
                    consequence: bound.into_from_env_goal(),
                    conditions: vec![
                        FromEnv::Trait(trait_ref.clone()).cast()
                    ],
                }
            }.cast()
        }));

        // add new type parameter U
        let mut binders = binders;
        binders.push(ParameterKind::Ty(()));
        let ty = Ty::Var(binders.len() - 1);

        // `Normalize(<T as Foo>::Assoc -> U)`
        let normalize = Normalize { projection: projection.clone(), ty: ty.clone() };

        // `ProjectionEq(<T as Foo>::Assoc = U)`
        let projection_eq = ProjectionEq { projection: projection.clone(), ty };

        // Projection equality rule from above.
        //
        //    forall<T, U> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :-
        //            Normalize(<T as Foo>::Assoc -> U).
        //    }
        clauses.push(Binders {
            binders: binders.clone(),
            value: ProgramClauseImplication {
                consequence: projection_eq.clone().cast(),
                conditions: vec![normalize.clone().cast()],
            },
        }.cast());

        clauses
    }
}
