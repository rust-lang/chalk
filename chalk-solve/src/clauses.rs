use self::builder::ClauseBuilder;
use self::env_elaborator::elaborate_env_clauses;
use self::program_clauses::ToProgramClauses;
use crate::goal_builder::GoalBuilder;
use crate::split::Split;
use crate::RustIrDatabase;
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use rustc_hash::FxHashSet;
use std::iter;
use tracing::{debug, instrument};

pub mod builder;
mod builtin_traits;
mod dyn_ty;
mod env_elaborator;
mod generalize;
pub mod program_clauses;

// yields the types "contained" in `app_ty`
fn constituent_types<I: Interner>(db: &dyn RustIrDatabase<I>, ty: &TyKind<I>) -> Vec<Ty<I>> {
    let interner = db.interner();

    match ty {
        // For non-phantom_data adts we collect its variants/fields
        TyKind::Adt(adt_id, substitution) if !db.adt_datum(*adt_id).flags.phantom_data => {
            let adt_datum = &db.adt_datum(*adt_id);
            let adt_datum_bound = adt_datum.binders.substitute(interner, substitution);
            adt_datum_bound
                .variants
                .into_iter()
                .flat_map(|variant| variant.fields.into_iter())
                .collect()
        }
        // And for `PhantomData<T>`, we pass `T`.
        TyKind::Adt(_, substitution)
        | TyKind::Tuple(_, substitution)
        | TyKind::FnDef(_, substitution) => substitution
            .iter(interner)
            .filter_map(|x| x.ty(interner))
            .cloned()
            .collect(),

        TyKind::Array(ty, _) | TyKind::Slice(ty) | TyKind::Raw(_, ty) | TyKind::Ref(_, _, ty) => {
            vec![ty.clone()]
        }

        TyKind::Str | TyKind::Never | TyKind::Scalar(_) => Vec::new(),

        TyKind::Generator(generator_id, substitution) => {
            let generator_datum = &db.generator_datum(*generator_id);
            let generator_datum_bound = generator_datum
                .input_output
                .substitute(interner, &substitution);

            let mut tys = generator_datum_bound.upvars;
            tys.push(
                TyKind::GeneratorWitness(*generator_id, substitution.clone()).intern(interner),
            );
            tys
        }

        TyKind::Closure(_, _) => panic!("this function should not be called for closures"),
        TyKind::GeneratorWitness(_, _) => {
            panic!("this function should not be called for generator witnesses")
        }
        TyKind::Function(_) => panic!("this function should not be called for functions"),
        TyKind::InferenceVar(_, _) | TyKind::BoundVar(_) => {
            panic!("this function should not be called for inference or bound vars")
        }
        TyKind::Placeholder(_) => panic!("this function should not be called for placeholders"),
        TyKind::Dyn(_) => panic!("this function should not be called for dyn types"),
        TyKind::Alias(_) => panic!("this function should not be called for alias"),
        TyKind::Foreign(_) => panic!("constituent_types of foreign types are unknown!"),
        TyKind::Error => Vec::new(),
        TyKind::OpaqueType(_, _) => unimplemented!(),
        TyKind::AssociatedType(_, _) => unimplemented!(),
    }
}

/// FIXME(#505) update comments for ADTs
/// For auto-traits, we generate a default rule for every struct,
/// unless there is a manual impl for that struct given explicitly.
///
/// So, if you have `impl Send for MyList<Foo>`, then we would
/// generate no rule for `MyList` at all -- similarly if you have
/// `impl !Send for MyList<Foo>`, or `impl<T> Send for MyList<T>`.
///
/// But if you have no rules at all for `Send` / `MyList`, then we
/// generate an impl based on the field types of `MyList`. For example
/// given the following program:
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
/// we generate:
///
/// ```notrust
/// forall<T> {
///     Implemented(MyList<T>: Send) :-
///         Implemented(T: Send),
///         Implemented(Box<Option<MyList<T>>>: Send).
/// }
/// ```
#[instrument(level = "debug", skip(builder))]
pub fn push_auto_trait_impls<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    auto_trait_id: TraitId<I>,
    ty: &TyKind<I>,
) -> Result<(), Floundered> {
    let interner = builder.interner();

    // Must be an auto trait.
    assert!(builder.db.trait_datum(auto_trait_id).is_auto_trait());

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(
        builder.db.trait_datum(auto_trait_id).binders.len(interner),
        1
    );

    // If there is a `impl AutoTrait for Foo<..>` or `impl !AutoTrait
    // for Foo<..>`, where `Foo` is the adt we're looking at, then
    // we don't generate our own rules.
    if builder.db.impl_provided_for(auto_trait_id, ty) {
        debug!("impl provided");
        return Ok(());
    }

    let mk_ref = |ty: Ty<I>| TraitRef {
        trait_id: auto_trait_id,
        substitution: Substitution::from1(interner, ty.cast(interner)),
    };

    let consequence = mk_ref(ty.clone().intern(interner));

    match ty {
        // function-types implement auto traits unconditionally
        TyKind::Function(_) => {
            let auto_trait_ref = TraitRef {
                trait_id: auto_trait_id,
                substitution: Substitution::from1(interner, ty.clone().intern(interner)),
            };

            builder.push_fact(auto_trait_ref);
            Ok(())
        }
        TyKind::InferenceVar(_, _) | TyKind::BoundVar(_) => Err(Floundered),

        // auto traits are not implemented for foreign types
        TyKind::Foreign(_) => Ok(()),

        // closures require binders, while the other types do not
        TyKind::Closure(closure_id, _) => {
            let binders = builder
                .db
                .closure_upvars(*closure_id, &Substitution::empty(interner));
            builder.push_binders(&binders, |builder, upvar_ty| {
                let conditions = iter::once(mk_ref(upvar_ty));
                builder.push_clause(consequence, conditions);
            });
            Ok(())
        }

        TyKind::GeneratorWitness(generator_id, _) => {
            push_auto_trait_impls_generator_witness(builder, auto_trait_id, *generator_id);
            Ok(())
        }

        // Unimplemented
        TyKind::OpaqueType(_, _) => Ok(()),
        TyKind::AssociatedType(_, _) => Ok(()),

        // No auto traits
        TyKind::Placeholder(_) | TyKind::Dyn(_) | TyKind::Alias(_) => Ok(()),

        // app_ty implements AutoTrait if all constituents of app_ty implement AutoTrait
        _ => {
            let conditions = constituent_types(builder.db, ty).into_iter().map(mk_ref);

            builder.push_clause(consequence, conditions);
            Ok(())
        }
    }
}

/// Leak auto traits for opaque types, just like `push_auto_trait_impls` does for structs.
///
/// For example, given the following program:
///
/// ```notrust
/// #[auto] trait Send { }
/// trait Trait { }
/// struct Bar { }
/// opaque type Foo: Trait = Bar
/// ```
/// Checking the goal `Foo: Send` would generate the following:
///
/// ```notrust
/// Foo: Send :- Bar: Send
/// ```
#[instrument(level = "debug", skip(builder))]
pub fn push_auto_trait_impls_opaque<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    auto_trait_id: TraitId<I>,
    opaque_id: OpaqueTyId<I>,
) {
    let opaque_ty_datum = &builder.db.opaque_ty_data(opaque_id);
    let interner = builder.interner();

    // Must be an auto trait.
    assert!(builder.db.trait_datum(auto_trait_id).is_auto_trait());

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(
        builder.db.trait_datum(auto_trait_id).binders.len(interner),
        1
    );

    let hidden_ty = builder.db.hidden_opaque_type(opaque_id);
    let binders = opaque_ty_datum.bound.clone();
    builder.push_binders(&binders, |builder, _| {
        let self_ty =
            TyKind::OpaqueType(opaque_id, builder.substitution_in_scope()).intern(interner);

        // trait_ref = `OpaqueType<...>: MyAutoTrait`
        let auto_trait_ref = TraitRef {
            trait_id: auto_trait_id,
            substitution: Substitution::from1(interner, self_ty),
        };

        // OpaqueType<...>: MyAutoTrait :- HiddenType: MyAutoTrait
        builder.push_clause(
            auto_trait_ref,
            std::iter::once(TraitRef {
                trait_id: auto_trait_id,
                substitution: Substitution::from1(interner, hidden_ty.clone()),
            }),
        );
    });
}

#[instrument(level = "debug", skip(builder))]
pub fn push_auto_trait_impls_generator_witness<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    auto_trait_id: TraitId<I>,
    generator_id: GeneratorId<I>,
) {
    let witness_datum = builder.db.generator_witness_datum(generator_id);
    let interner = builder.interner();

    // Must be an auto trait.
    assert!(builder.db.trait_datum(auto_trait_id).is_auto_trait());

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(
        builder.db.trait_datum(auto_trait_id).binders.len(interner),
        1
    );

    // Push binders for the generator generic parameters. These can be used by
    // both upvars and witness types
    builder.push_binders(&witness_datum.inner_types, |builder, inner_types| {
        let witness_ty = TyKind::GeneratorWitness(generator_id, builder.substitution_in_scope())
            .intern(interner);

        // trait_ref = `GeneratorWitness<...>: MyAutoTrait`
        let auto_trait_ref = TraitRef {
            trait_id: auto_trait_id,
            substitution: Substitution::from1(interner, witness_ty),
        };

        // Create a goal of the form:
        // forall<L0, L1, ..., LN> {
        //     WitnessType1<L0, L1, ... LN, P0, P1, ..., PN>: MyAutoTrait,
        //     ...
        //     WitnessTypeN<L0, L1, ... LN, P0, P1, ..., PN>: MyAutoTrait,
        //
        // }
        //
        // where `L0, L1, ...LN` are our existentially bound witness lifetimes,
        // and `P0, P1, ..., PN` are the normal generator generics.
        //
        // We create a 'forall' goal due to the fact that our witness lifetimes
        // are *existentially* quantified - the precise reigon is erased during
        // type checking, so we just know that the type takes *some* region
        // as a parameter. Therefore, we require that the auto trait bound
        // hold for *all* regions, which guarantees that the bound will
        // hold for the original lifetime (before it was erased).
        //
        // This does not take into account well-formed information from
        // the witness types. For example, if we have the type
        // `struct Foo<'a, 'b> { val: &'a &'b u8 }`
        // then `'b: 'a` must hold for `Foo<'a, 'b>` to be well-formed.
        // If we have `Foo<'a, 'b>` stored as a witness type, we will
        // not currently use this information to determine a more precise
        // relationship between 'a and 'b. In the future, we will likely
        // do this to avoid incorrectly rejecting correct code.
        let gb = &mut GoalBuilder::new(builder.db);
        let witness_goal = gb.forall(
            &inner_types.types,
            auto_trait_id,
            |gb, _subst, types, auto_trait_id| {
                Goal::new(
                    gb.interner(),
                    GoalData::All(Goals::from_iter(
                        gb.interner(),
                        types.iter().map(|witness_ty| TraitRef {
                            trait_id: auto_trait_id,
                            substitution: Substitution::from1(gb.interner(), witness_ty.clone()),
                        }),
                    )),
                )
            },
        );

        // GeneratorWitnessType: AutoTrait :- forall<...> ...
        // where 'forall<...> ...' is the goal described above.
        builder.push_clause(auto_trait_ref, std::iter::once(witness_goal));
    })
}

/// Given some goal `goal` that must be proven, along with
/// its `environment`, figures out the program clauses that apply
/// to this goal from the Rust program. So for example if the goal
/// is `Implemented(T: Clone)`, then this function might return clauses
/// derived from the trait `Clone` and its impls.
#[instrument(level = "debug", skip(db))]
pub fn program_clauses_for_goal<'db, I: Interner>(
    db: &'db dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    goal: &DomainGoal<I>,
    binders: &CanonicalVarKinds<I>,
) -> Result<Vec<ProgramClause<I>>, Floundered> {
    let interner = db.interner();

    let custom_clauses = db.custom_clauses().into_iter();
    let clauses_that_could_match = program_clauses_that_could_match(db, environment, goal, binders)
        .map(|cl| cl.into_iter())?;

    let clauses: Vec<ProgramClause<I>> = custom_clauses
        .chain(clauses_that_could_match)
        .chain(
            db.program_clauses_for_env(environment)
                .iter(interner)
                .cloned(),
        )
        .filter(|c| c.could_match(interner, goal))
        .collect();

    debug!(?clauses);

    Ok(clauses)
}

/// Returns a set of program clauses that could possibly match
/// `goal`. This can be any superset of the correct set, but the
/// more precise you can make it, the more efficient solving will
/// be.
#[instrument(level = "debug", skip(db, environment))]
fn program_clauses_that_could_match<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    goal: &DomainGoal<I>,
    // FIXME: These are the binders for `goal`. We're passing them separately
    // because `goal` is not necessarily canonicalized: The recursive solver
    // passes the canonical goal; the SLG solver instantiates the goal first.
    // (See #568.)
    binders: &CanonicalVarKinds<I>,
) -> Result<Vec<ProgramClause<I>>, Floundered> {
    let interner = db.interner();
    let mut clauses: Vec<ProgramClause<I>> = vec![];
    let builder = &mut ClauseBuilder::new(db, &mut clauses);

    match goal {
        DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
            let trait_id = trait_ref.trait_id;

            let trait_datum = db.trait_datum(trait_id);

            if trait_datum.is_non_enumerable_trait() || trait_datum.is_auto_trait() {
                let self_ty = trait_ref.self_type_parameter(interner);

                if let TyKind::Alias(AliasTy::Opaque(opaque_ty)) = self_ty.kind(interner) {
                    if trait_datum.is_auto_trait() {
                        push_auto_trait_impls_opaque(builder, trait_id, opaque_ty.opaque_ty_id)
                    }
                } else if self_ty.is_general_var(interner, binders) {
                    return Err(Floundered);
                }
            }

            // This is needed for the coherence related impls, as well
            // as for the `Implemented(Foo) :- FromEnv(Foo)` rule.
            trait_datum.to_program_clauses(builder, environment);

            for impl_id in db.impls_for_trait(
                trait_ref.trait_id,
                trait_ref.substitution.as_slice(interner),
                binders,
            ) {
                db.impl_datum(impl_id)
                    .to_program_clauses(builder, environment);
            }

            // If this is a `Foo: Send` (or any auto-trait), then add
            // the automatic impls for `Foo`.
            let trait_datum = db.trait_datum(trait_id);
            if trait_datum.is_auto_trait() {
                let generalized = generalize::Generalize::apply(db.interner(), trait_ref);
                builder.push_binders(&generalized, |builder, trait_ref| {
                    let ty = trait_ref.self_type_parameter(interner);
                    push_auto_trait_impls(builder, trait_id, &ty.kind(interner))
                })?;
            }

            // If the self type is a `dyn trait` type, generate program-clauses
            // that indicates that it implements its own traits.
            // FIXME: This is presently rather wasteful, in that we don't check that the
            // these program clauses we are generating are actually relevant to the goal
            // `goal` that we are actually *trying* to prove (though there is some later
            // code that will screen out irrelevant stuff).
            //
            // In other words, if we were trying to prove `Implemented(dyn
            // Fn(&u8): Clone)`, we would still generate two clauses that are
            // totally irrelevant to that goal, because they let us prove other
            // things but not `Clone`.
            let self_ty = trait_ref.self_type_parameter(interner);
            if let TyKind::Dyn(_) = self_ty.kind(interner) {
                dyn_ty::build_dyn_self_ty_clauses(db, builder, self_ty.clone())
            }

            match self_ty.kind(interner) {
                TyKind::OpaqueType(opaque_ty_id, _)
                | TyKind::Alias(AliasTy::Opaque(OpaqueTy { opaque_ty_id, .. })) => {
                    db.opaque_ty_data(*opaque_ty_id)
                        .to_program_clauses(builder, environment);
                }
                _ => {}
            }

            // We don't actually do anything here, but we need to record the types it when logging
            match self_ty.kind(interner) {
                TyKind::Adt(adt_id, _) => {
                    let _ = db.adt_datum(*adt_id);
                }
                TyKind::FnDef(fn_def_id, _) => {
                    let _ = db.fn_def_datum(*fn_def_id);
                }
                _ => {}
            }

            if let Some(well_known) = trait_datum.well_known {
                builtin_traits::add_builtin_program_clauses(
                    db, builder, well_known, trait_ref, binders,
                )?;
            }
        }
        DomainGoal::Holds(WhereClause::AliasEq(alias_eq)) => match &alias_eq.alias {
            AliasTy::Projection(proj) => {
                let trait_self_ty = db
                    .trait_ref_from_projection(proj)
                    .self_type_parameter(interner);

                match trait_self_ty.kind(interner) {
                    TyKind::OpaqueType(opaque_ty_id, _)
                    | TyKind::Alias(AliasTy::Opaque(OpaqueTy { opaque_ty_id, .. })) => {
                        db.opaque_ty_data(*opaque_ty_id)
                            .to_program_clauses(builder, environment);
                    }
                    _ => {}
                }

                // If the self type is a `dyn trait` type, generate program-clauses
                // for any associated type bindings it contains.
                // FIXME: see the fixme for the analogous code for Implemented goals.
                if let TyKind::Dyn(_) = trait_self_ty.kind(interner) {
                    dyn_ty::build_dyn_self_ty_clauses(db, builder, trait_self_ty.clone())
                }

                db.associated_ty_data(proj.associated_ty_id)
                    .to_program_clauses(builder, environment)
            }
            AliasTy::Opaque(opaque_ty) => db
                .opaque_ty_data(opaque_ty.opaque_ty_id)
                .to_program_clauses(builder, environment),
        },
        DomainGoal::Holds(WhereClause::LifetimeOutlives(..)) => {
            builder.push_bound_lifetime(|builder, a| {
                builder.push_bound_lifetime(|builder, b| {
                    builder.push_fact_with_constraints(
                        DomainGoal::Holds(WhereClause::LifetimeOutlives(LifetimeOutlives {
                            a: a.clone(),
                            b: b.clone(),
                        })),
                        Some(InEnvironment::new(
                            environment,
                            Constraint::LifetimeOutlives(a, b),
                        )),
                    );
                })
            });
        }
        DomainGoal::Holds(WhereClause::TypeOutlives(..)) => {
            builder.push_bound_ty(|builder, ty| {
                builder.push_bound_lifetime(|builder, lifetime| {
                    builder.push_fact_with_constraints(
                        DomainGoal::Holds(WhereClause::TypeOutlives(TypeOutlives {
                            ty: ty.clone(),
                            lifetime: lifetime.clone(),
                        })),
                        Some(InEnvironment::new(
                            environment,
                            Constraint::TypeOutlives(ty, lifetime),
                        )),
                    )
                })
            });
        }
        DomainGoal::WellFormed(WellFormed::Trait(trait_ref))
        | DomainGoal::LocalImplAllowed(trait_ref) => {
            db.trait_datum(trait_ref.trait_id)
                .to_program_clauses(builder, environment);
        }
        DomainGoal::ObjectSafe(trait_id) => {
            if builder.db.is_object_safe(*trait_id) {
                builder.push_fact(DomainGoal::ObjectSafe(*trait_id));
            }
        }
        DomainGoal::WellFormed(WellFormed::Ty(ty))
        | DomainGoal::IsUpstream(ty)
        | DomainGoal::DownstreamType(ty)
        | DomainGoal::IsFullyVisible(ty)
        | DomainGoal::IsLocal(ty) => match_ty(builder, environment, ty)?,
        DomainGoal::FromEnv(_) => (), // Computed in the environment
        DomainGoal::Normalize(Normalize { alias, ty: _ }) => match alias {
            AliasTy::Projection(proj) => {
                // Normalize goals derive from `AssociatedTyValue` datums,
                // which are found in impls. That is, if we are
                // normalizing (e.g.) `<T as Iterator>::Item>`, then
                // search for impls of iterator and, within those impls,
                // for associated type values:
                //
                // ```ignore
                // impl Iterator for Foo {
                //     type Item = Bar; // <-- associated type value
                // }
                // ```
                let associated_ty_datum = db.associated_ty_data(proj.associated_ty_id);
                let trait_id = associated_ty_datum.trait_id;
                let trait_parameters = db.trait_parameters_from_projection(proj);

                let trait_datum = db.trait_datum(trait_id);

                let self_ty = alias.self_type_parameter(interner);

                // Flounder if the self-type is unknown and the trait is non-enumerable.
                //
                // e.g., Normalize(<?X as Iterator>::Item = u32)
                if (self_ty.is_general_var(interner, binders))
                    && trait_datum.is_non_enumerable_trait()
                {
                    return Err(Floundered);
                }

                if let Some(well_known) = trait_datum.well_known {
                    builtin_traits::add_builtin_assoc_program_clauses(
                        db, builder, well_known, self_ty,
                    )?;
                }

                push_program_clauses_for_associated_type_values_in_impls_of(
                    builder,
                    environment,
                    trait_id,
                    trait_parameters,
                    binders,
                );

                if environment.has_compatible_clause(interner) {
                    push_clauses_for_compatible_normalize(
                        db,
                        builder,
                        interner,
                        trait_id,
                        proj.associated_ty_id,
                    );
                }
            }
            AliasTy::Opaque(_) => (),
        },
        DomainGoal::Compatible | DomainGoal::Reveal => (),
    };

    Ok(clauses)
}

/// Adds clauses to allow normalizing possible downstream associated type
/// implementations when in the "compatible" mode. Example clauses:
///
/// ```notrust
/// for<type, type, type> Normalize(<^0.0 as Trait<^0.1>>::Item -> ^0.2)
///     :- Compatible, Implemented(^0.0: Trait<^0.1>), DownstreamType(^0.1), CannotProve
/// for<type, type, type> Normalize(<^0.0 as Trait<^0.1>>::Item -> ^0.2)
///     :- Compatible, Implemented(^0.0: Trait<^0.1>), IsFullyVisible(^0.0), DownstreamType(^0.1), CannotProve
/// ```
fn push_clauses_for_compatible_normalize<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    interner: &I,
    trait_id: TraitId<I>,
    associated_ty_id: AssocTypeId<I>,
) {
    let trait_datum = db.trait_datum(trait_id);
    let trait_binders = trait_datum.binders.map_ref(|b| &b.where_clauses);
    builder.push_binders(&trait_binders, |builder, where_clauses| {
        let projection = ProjectionTy {
            associated_ty_id,
            substitution: builder.substitution_in_scope(),
        };
        let trait_ref = TraitRef {
            trait_id,
            substitution: builder.substitution_in_scope(),
        };
        let type_parameters: Vec<_> = trait_ref.type_parameters(interner).collect();

        builder.push_bound_ty(|builder, target_ty| {
            for i in 0..type_parameters.len() {
                builder.push_clause(
                    DomainGoal::Normalize(Normalize {
                        ty: target_ty.clone(),
                        alias: AliasTy::Projection(projection.clone()),
                    }),
                    where_clauses
                        .iter()
                        .cloned()
                        .casted(interner)
                        .chain(iter::once(DomainGoal::Compatible.cast(interner)))
                        .chain(iter::once(
                            WhereClause::Implemented(trait_ref.clone()).cast(interner),
                        ))
                        .chain((0..i).map(|j| {
                            DomainGoal::IsFullyVisible(type_parameters[j].clone()).cast(interner)
                        }))
                        .chain(iter::once(
                            DomainGoal::DownstreamType(type_parameters[i].clone()).cast(interner),
                        ))
                        .chain(iter::once(GoalData::CannotProve.intern(interner))),
                );
            }
        });
    });
}

/// Generate program clauses from the associated-type values
/// found in impls of the given trait. i.e., if `trait_id` = Iterator,
/// then we would generate program clauses from each `type Item = ...`
/// found in any impls of `Iterator`:
/// which are found in impls. That is, if we are
/// normalizing (e.g.) `<T as Iterator>::Item>`, then
/// search for impls of iterator and, within those impls,
/// for associated type values:
///
/// ```ignore
/// impl Iterator for Foo {
///     type Item = Bar; // <-- associated type value
/// }
/// ```
#[instrument(level = "debug", skip(builder))]
fn push_program_clauses_for_associated_type_values_in_impls_of<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    environment: &Environment<I>,
    trait_id: TraitId<I>,
    trait_parameters: &[GenericArg<I>],
    binders: &CanonicalVarKinds<I>,
) {
    for impl_id in builder
        .db
        .impls_for_trait(trait_id, trait_parameters, binders)
    {
        let impl_datum = builder.db.impl_datum(impl_id);
        if !impl_datum.is_positive() {
            continue;
        }

        debug!(?impl_id);

        for &atv_id in &impl_datum.associated_ty_value_ids {
            let atv = builder.db.associated_ty_value(atv_id);
            debug!(?atv_id, ?atv);
            atv.to_program_clauses(builder, environment);
        }
    }
}

/// Examine `T` and push clauses that may be relevant to proving the
/// following sorts of goals (and maybe others):
///
/// * `DomainGoal::WellFormed(T)`
/// * `DomainGoal::IsUpstream(T)`
/// * `DomainGoal::DownstreamType(T)`
/// * `DomainGoal::IsFullyVisible(T)`
/// * `DomainGoal::IsLocal(T)`
///
/// Note that the type `T` must not be an unbound inference variable;
/// earlier parts of the logic should "flounder" in that case.
fn match_ty<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    environment: &Environment<I>,
    ty: &Ty<I>,
) -> Result<(), Floundered> {
    let interner = builder.interner();
    Ok(match ty.kind(interner) {
        TyKind::Adt(adt_id, _) => builder
            .db
            .adt_datum(*adt_id)
            .to_program_clauses(builder, environment),
        TyKind::OpaqueType(opaque_ty_id, _) => builder
            .db
            .opaque_ty_data(*opaque_ty_id)
            .to_program_clauses(builder, environment),
        TyKind::Error => {}
        TyKind::AssociatedType(type_id, _) => builder
            .db
            .associated_ty_data(*type_id)
            .to_program_clauses(builder, environment),
        TyKind::FnDef(fn_def_id, _) => builder
            .db
            .fn_def_datum(*fn_def_id)
            .to_program_clauses(builder, environment),
        TyKind::Tuple(_, _)
        | TyKind::Scalar(_)
        | TyKind::Str
        | TyKind::Slice(_)
        | TyKind::Raw(_, _)
        | TyKind::Ref(_, _, _)
        | TyKind::Array(_, _)
        | TyKind::Never
        | TyKind::Closure(_, _)
        | TyKind::Foreign(_)
        | TyKind::Generator(_, _)
        | TyKind::GeneratorWitness(_, _) => builder.push_fact(WellFormed::Ty(ty.clone())),
        TyKind::Placeholder(_) => {
            builder.push_clause(WellFormed::Ty(ty.clone()), Some(FromEnv::Ty(ty.clone())));
        }
        TyKind::Alias(AliasTy::Projection(proj)) => builder
            .db
            .associated_ty_data(proj.associated_ty_id)
            .to_program_clauses(builder, environment),
        TyKind::Alias(AliasTy::Opaque(opaque_ty)) => builder
            .db
            .opaque_ty_data(opaque_ty.opaque_ty_id)
            .to_program_clauses(builder, environment),
        TyKind::Function(_quantified_ty) => {
            builder.push_fact(WellFormed::Ty(ty.clone()));
        }
        TyKind::BoundVar(_) | TyKind::InferenceVar(_, _) => return Err(Floundered),
        TyKind::Dyn(_) => {}
    })
}

fn match_alias_ty<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    environment: &Environment<I>,
    alias: &AliasTy<I>,
) {
    match alias {
        AliasTy::Projection(projection_ty) => builder
            .db
            .associated_ty_data(projection_ty.associated_ty_id)
            .to_program_clauses(builder, environment),
        _ => (),
    }
}

pub fn program_clauses_for_env<'db, I: Interner>(
    db: &'db dyn RustIrDatabase<I>,
    environment: &Environment<I>,
) -> ProgramClauses<I> {
    let mut last_round = environment
        .clauses
        .as_slice(db.interner())
        .iter()
        .cloned()
        .collect::<FxHashSet<_>>();
    let mut closure = last_round.clone();
    let mut next_round = FxHashSet::default();
    while !last_round.is_empty() {
        elaborate_env_clauses(
            db,
            &last_round.drain().collect::<Vec<_>>(),
            &mut next_round,
            environment,
        );
        last_round.extend(
            next_round
                .drain()
                .filter(|clause| closure.insert(clause.clone())),
        );
    }

    ProgramClauses::from_iter(db.interner(), closure)
}
