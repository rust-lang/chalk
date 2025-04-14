use self::builder::ClauseBuilder;
use self::env_elaborator::elaborate_env_clauses;
use self::program_clauses::ToProgramClauses;
use crate::goal_builder::GoalBuilder;
use crate::rust_ir::{Movability, WellKnownTrait};
use crate::split::Split;
use crate::RustIrDatabase;
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use rustc_hash::FxHashSet;
use std::iter;
use std::marker::PhantomData;
use tracing::{debug, instrument};

pub mod builder;
mod builtin_traits;
mod dyn_ty;
mod env_elaborator;
mod generalize;
pub mod program_clauses;
mod super_traits;

// yields the types "contained" in `app_ty`
fn constituent_types<I: Interner>(db: &dyn RustIrDatabase<I>, ty: &TyKind<I>) -> Vec<Ty<I>> {
    let interner = db.interner();

    match ty {
        // For non-phantom_data adts we collect its variants/fields
        TyKind::Adt(adt_id, substitution) if !db.adt_datum(*adt_id).flags.phantom_data => {
            let adt_datum = &db.adt_datum(*adt_id);
            let adt_datum_bound = adt_datum.binders.clone().substitute(interner, substitution);
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

        TyKind::Coroutine(coroutine_id, substitution) => {
            let coroutine_datum = &db.coroutine_datum(*coroutine_id);
            let coroutine_datum_bound = coroutine_datum
                .input_output
                .clone()
                .substitute(interner, &substitution);

            let mut tys = coroutine_datum_bound.upvars;
            tys.push(
                TyKind::CoroutineWitness(*coroutine_id, substitution.clone()).intern(interner),
            );
            tys
        }

        TyKind::Closure(_, _) => panic!("this function should not be called for closures"),
        TyKind::CoroutineWitness(_, _) => {
            panic!("this function should not be called for coroutine witnesses")
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
        TyKind::OpaqueType(_, _) => panic!("constituent_types of opaque types are unknown!"),
        TyKind::AssociatedType(_, _) => {
            panic!("constituent_types of associated types are unknown!")
        }
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
            builder.push_fact(consequence);
            Ok(())
        }
        TyKind::InferenceVar(_, _) | TyKind::BoundVar(_) => Err(Floundered),

        // auto traits are not implemented for foreign types
        TyKind::Foreign(_) => Ok(()),

        // closures require binders, while the other types do not
        TyKind::Closure(closure_id, substs) => {
            let closure_fn_substitution = builder.db.closure_fn_substitution(*closure_id, substs);
            let binders = builder.db.closure_upvars(*closure_id, substs);
            let upvars = binders.substitute(builder.db.interner(), &closure_fn_substitution);

            // in a same behavior as for non-auto traits (reuse the code) we can require that
            // every bound type must implement this auto-trait
            use crate::clauses::builtin_traits::needs_impl_for_tys;
            needs_impl_for_tys(builder.db, builder, consequence, Some(upvars).into_iter());

            Ok(())
        }
        TyKind::Coroutine(coroutine_id, _) => {
            if Some(auto_trait_id) == builder.db.well_known_trait_id(WellKnownTrait::Unpin) {
                match builder.db.coroutine_datum(*coroutine_id).movability {
                    // immovable coroutines are never `Unpin`
                    Movability::Static => (),
                    // movable coroutines are always `Unpin`
                    Movability::Movable => builder.push_fact(consequence),
                }
            } else {
                // if trait is not `Unpin`, use regular auto trait clause
                let conditions = constituent_types(builder.db, ty).into_iter().map(mk_ref);
                builder.push_clause(consequence, conditions);
            }
            Ok(())
        }

        TyKind::CoroutineWitness(coroutine_id, _) => {
            push_auto_trait_impls_coroutine_witness(builder, auto_trait_id, *coroutine_id);
            Ok(())
        }

        TyKind::OpaqueType(opaque_ty_id, _) => {
            push_auto_trait_impls_opaque(builder, auto_trait_id, *opaque_ty_id);
            Ok(())
        }

        // No auto traits
        TyKind::AssociatedType(_, _)
        | TyKind::Placeholder(_)
        | TyKind::Dyn(_)
        | TyKind::Alias(_) => Ok(()),

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
    builder.push_binders(binders, |builder, _| {
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
pub fn push_auto_trait_impls_coroutine_witness<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    auto_trait_id: TraitId<I>,
    coroutine_id: CoroutineId<I>,
) {
    let witness_datum = builder.db.coroutine_witness_datum(coroutine_id);
    let interner = builder.interner();

    // Must be an auto trait.
    assert!(builder.db.trait_datum(auto_trait_id).is_auto_trait());

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(
        builder.db.trait_datum(auto_trait_id).binders.len(interner),
        1
    );

    // Push binders for the coroutine generic parameters. These can be used by
    // both upvars and witness types
    builder.push_binders(witness_datum.inner_types.clone(), |builder, inner_types| {
        let witness_ty = TyKind::CoroutineWitness(coroutine_id, builder.substitution_in_scope())
            .intern(interner);

        // trait_ref = `CoroutineWitness<...>: MyAutoTrait`
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
        // and `P0, P1, ..., PN` are the normal coroutine generics.
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

        // CoroutineWitnessType: AutoTrait :- forall<...> ...
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
    goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
) -> Result<Vec<ProgramClause<I>>, Floundered> {
    let interner = db.interner();

    let custom_clauses = db.custom_clauses().into_iter();
    let clauses_that_could_match =
        program_clauses_that_could_match(db, goal).map(|cl| cl.into_iter())?;

    let clauses: Vec<ProgramClause<I>> = custom_clauses
        .chain(clauses_that_could_match)
        .chain(
            db.program_clauses_for_env(&goal.canonical.value.environment)
                .iter(interner)
                .cloned(),
        )
        .filter(|c| {
            c.could_match(
                interner,
                db.unification_database(),
                &goal.canonical.value.goal,
            )
        })
        .collect();

    debug!(?clauses);

    Ok(clauses)
}

/// Returns a set of program clauses that could possibly match
/// `goal`. This can be any superset of the correct set, but the
/// more precise you can make it, the more efficient solving will
/// be.
#[instrument(level = "debug", skip(db))]
pub fn program_clauses_that_could_match<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    goal: &UCanonical<InEnvironment<DomainGoal<I>>>,
) -> Result<Vec<ProgramClause<I>>, Floundered> {
    let interner = db.interner();
    let mut clauses: Vec<ProgramClause<I>> = vec![];
    let builder = &mut ClauseBuilder::new(db, &mut clauses);

    let UCanonical {
        canonical:
            Canonical {
                value: InEnvironment { environment, goal },
                binders,
            },
        universes: _,
    } = goal;

    match goal {
        DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
            let self_ty = trait_ref.self_type_parameter(interner);

            let trait_id = trait_ref.trait_id;
            let trait_datum = db.trait_datum(trait_id);

            match self_ty.kind(interner) {
                TyKind::InferenceVar(_, _) => {
                    panic!("Inference vars not allowed when getting program clauses")
                }
                TyKind::Alias(alias) => {
                    // An alias could normalize to anything, including `dyn trait`
                    // or an opaque type, so push a clause that asks for the
                    // self type to be normalized and return.
                    push_alias_implemented_clause(builder, trait_ref.clone(), alias.clone());
                    return Ok(clauses);
                }

                _ if self_ty.is_general_var(interner, binders) => {
                    if trait_datum.is_non_enumerable_trait() || trait_datum.is_auto_trait() {
                        return Err(Floundered);
                    }
                }

                TyKind::OpaqueType(opaque_ty_id, _) => {
                    db.opaque_ty_data(*opaque_ty_id)
                        .to_program_clauses(builder, environment);
                }

                TyKind::AssociatedType(assoc_ty_id, _) => {
                    db.associated_ty_data(*assoc_ty_id)
                        .to_program_clauses(builder, environment);
                }

                TyKind::Dyn(_) => {
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
                    dyn_ty::build_dyn_self_ty_clauses(db, builder, self_ty.clone())
                }

                // We don't actually do anything here, but we need to record the types when logging
                TyKind::Adt(adt_id, _) => {
                    let _ = db.adt_datum(*adt_id);
                }

                TyKind::FnDef(fn_def_id, _) => {
                    let _ = db.fn_def_datum(*fn_def_id);
                }

                _ => {}
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
                let generalized = generalize::Generalize::apply(db.interner(), trait_ref.clone());
                builder.push_binders(generalized, |builder, trait_ref| {
                    let ty = trait_ref.self_type_parameter(interner);
                    push_auto_trait_impls(builder, trait_id, ty.kind(interner))
                })?;
            }

            if let Some(well_known) = trait_datum.well_known {
                builtin_traits::add_builtin_program_clauses(
                    db,
                    builder,
                    well_known,
                    trait_ref.clone(),
                    binders,
                )?;
            }
        }
        DomainGoal::Holds(WhereClause::AliasEq(alias_eq)) => match &alias_eq.alias {
            AliasTy::Projection(proj) => {
                let trait_self_ty = db
                    .trait_ref_from_projection(proj)
                    .self_type_parameter(interner);

                match trait_self_ty.kind(interner) {
                    TyKind::Alias(alias) => {
                        // An alias could normalize to anything, including an
                        // opaque type, so push a clause that asks for the self
                        // type to be normalized and return.
                        push_alias_alias_eq_clause(builder, proj.clone(), alias.clone());
                        return Ok(clauses);
                    }
                    TyKind::OpaqueType(opaque_ty_id, _) => {
                        db.opaque_ty_data(*opaque_ty_id)
                            .to_program_clauses(builder, environment);
                    }
                    TyKind::AssociatedType(assoc_ty_id, _) => {
                        db.associated_ty_data(*assoc_ty_id)
                            .to_program_clauses(builder, environment);
                    }
                    // If the self type is a `dyn trait` type, generate program-clauses
                    // for any associated type bindings it contains.
                    // FIXME: see the fixme for the analogous code for Implemented goals.
                    TyKind::Dyn(_) => {
                        dyn_ty::build_dyn_self_ty_clauses(db, builder, trait_self_ty.clone())
                    }
                    _ => {}
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
                            &Environment::new(interner),
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
                            &Environment::new(interner),
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
                let trait_ref = db.trait_ref_from_projection(proj);
                let trait_parameters = trait_ref.substitution.as_parameters(interner);

                let trait_datum = db.trait_datum(trait_id);

                let self_ty = trait_ref.self_type_parameter(interner);
                if let TyKind::InferenceVar(_, _) = self_ty.kind(interner) {
                    panic!("Inference vars not allowed when getting program clauses");
                }

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
                        db,
                        builder,
                        well_known,
                        self_ty.clone(),
                    )?;
                }

                push_program_clauses_for_associated_type_values_in_impls_of(
                    builder,
                    environment,
                    trait_id,
                    proj.associated_ty_id,
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

                // When `self_ty` is dyn type or opaque type, there may be associated type bounds
                // for which we generate `Normalize` clauses.
                match self_ty.kind(interner) {
                    // FIXME: see the fixme for the analogous code for Implemented goals.
                    TyKind::Dyn(_) => dyn_ty::build_dyn_self_ty_clauses(db, builder, self_ty),
                    TyKind::OpaqueType(id, _) => {
                        db.opaque_ty_data(*id)
                            .to_program_clauses(builder, environment);
                    }
                    _ => {}
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
    interner: I,
    trait_id: TraitId<I>,
    associated_ty_id: AssocTypeId<I>,
) {
    let trait_datum = db.trait_datum(trait_id);
    let trait_binders = trait_datum.binders.map_ref(|b| &b.where_clauses).cloned();
    builder.push_binders(trait_binders, |builder, where_clauses| {
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
    assoc_id: AssocTypeId<I>,
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

        if let Some(atv_id) = builder.db.associated_ty_from_impl(impl_id, assoc_id) {
            let atv = builder.db.associated_ty_value(atv_id);
            debug!(?atv_id, ?atv);
            atv.to_program_clauses(builder, environment);
        }
    }
}

fn push_alias_implemented_clause<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
    alias: AliasTy<I>,
) {
    let interner = builder.interner();
    assert_eq!(
        *trait_ref.self_type_parameter(interner).kind(interner),
        TyKind::Alias(alias.clone())
    );

    // TODO: instead generate clauses without reference to the specific type parameters of the goal?
    let generalized = generalize::Generalize::apply(interner, (trait_ref, alias));
    builder.push_binders(generalized, |builder, (trait_ref, alias)| {
        // forall<..., T> {
        //      <X as Y>::Z: Trait :- T: Trait, <X as Y>::Z == T
        // }
        builder.push_bound_ty(|builder, bound_var| {
            let fresh_self_subst = Substitution::from_iter(
                interner,
                std::iter::once(bound_var.clone().cast(interner)).chain(
                    trait_ref.substitution.as_slice(interner)[1..]
                        .iter()
                        .cloned(),
                ),
            );
            let fresh_self_trait_ref = TraitRef {
                trait_id: trait_ref.trait_id,
                substitution: fresh_self_subst,
            };
            builder.push_clause(
                DomainGoal::Holds(WhereClause::Implemented(trait_ref.clone())),
                &[
                    DomainGoal::Holds(WhereClause::Implemented(fresh_self_trait_ref)),
                    DomainGoal::Holds(WhereClause::AliasEq(AliasEq {
                        alias: alias.clone(),
                        ty: bound_var,
                    })),
                ],
            );
        });
    });
}

fn push_alias_alias_eq_clause<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    projection_ty: ProjectionTy<I>,
    alias: AliasTy<I>,
) {
    let interner = builder.interner();
    let self_ty = builder
        .db
        .trait_ref_from_projection(&projection_ty)
        .self_type_parameter(interner);
    assert_eq!(*self_ty.kind(interner), TyKind::Alias(alias.clone()));

    // TODO: instead generate clauses without reference to the specific type parameters of the goal?
    let generalized = generalize::Generalize::apply(interner, (projection_ty, alias));
    builder.push_binders(generalized, |builder, (projection_ty, alias)| {
        // Given the following canonical goal:
        //
        // ```
        // forall<...> {
        //     <<X as Y>::A as Z>::B == W
        // }
        // ```
        //
        // we generate:
        //
        // ```
        // forall<..., T, U> {
        //      <<X as Y>::A as Z>::B == U :- <T as Z>::B == U, <X as Y>::A == T
        // }
        // ```
        //
        // `T` and `U` are `intermediate_eq_ty` and `eq_ty` respectively below.
        //
        // Note that we used to "reuse" `W` and push:
        //
        // ```
        // forall<..., T> {
        //      <<X as Y>::A as Z>::B == W :- <T as Z>::B == W, <X as Y>::A == T
        // }
        // ```
        //
        // but it caused a cycle which led to false `NoSolution` under certain conditions, in
        // particular when `W` itself is a nested projection type. See test
        // `nested_proj_eq_nested_proj_should_flounder`.
        builder.push_bound_ty(|builder, intermediate_eq_ty| {
            builder.push_bound_ty(|builder, eq_ty| {
                let (_, trait_args, assoc_args) = builder.db.split_projection(&projection_ty);
                let fresh_self_subst = Substitution::from_iter(
                    interner,
                    std::iter::once(intermediate_eq_ty.clone().cast(interner))
                        .chain(trait_args[1..].iter().cloned())
                        .chain(assoc_args.iter().cloned()),
                );
                let fresh_alias = AliasTy::Projection(ProjectionTy {
                    associated_ty_id: projection_ty.associated_ty_id,
                    substitution: fresh_self_subst,
                });
                builder.push_clause(
                    DomainGoal::Holds(WhereClause::AliasEq(AliasEq {
                        alias: AliasTy::Projection(projection_ty.clone()),
                        ty: eq_ty.clone(),
                    })),
                    &[
                        DomainGoal::Holds(WhereClause::AliasEq(AliasEq {
                            alias: fresh_alias,
                            ty: eq_ty,
                        })),
                        DomainGoal::Holds(WhereClause::AliasEq(AliasEq {
                            alias,
                            ty: intermediate_eq_ty,
                        })),
                    ],
                );
            });
        });
    });
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
    match ty.kind(interner) {
        TyKind::InferenceVar(_, _) => {
            panic!("Inference vars not allowed when getting program clauses")
        }
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
        TyKind::Str
        | TyKind::Never
        | TyKind::Scalar(_)
        | TyKind::Foreign(_)
        | TyKind::Tuple(0, _) => {
            // These have no substitutions, so they are trivially WF
            builder.push_fact(WellFormed::Ty(ty.clone()));
        }
        TyKind::Raw(mutbl, _) => {
            // forall<T> WF(*const T) :- WF(T);
            builder.push_bound_ty(|builder, ty| {
                builder.push_clause(
                    WellFormed::Ty(TyKind::Raw(*mutbl, ty.clone()).intern(builder.interner())),
                    Some(WellFormed::Ty(ty)),
                );
            });
        }
        TyKind::Ref(mutbl, _, _) => {
            // forall<'a, T> WF(&'a T) :- WF(T), T: 'a
            builder.push_bound_ty(|builder, ty| {
                builder.push_bound_lifetime(|builder, lifetime| {
                    let ref_ty = TyKind::Ref(*mutbl, lifetime.clone(), ty.clone())
                        .intern(builder.interner());
                    builder.push_clause(
                        WellFormed::Ty(ref_ty),
                        [
                            DomainGoal::WellFormed(WellFormed::Ty(ty.clone())),
                            DomainGoal::Holds(WhereClause::TypeOutlives(TypeOutlives {
                                ty,
                                lifetime,
                            })),
                        ],
                    );
                })
            });
        }
        TyKind::Slice(_) => {
            // forall<T> WF([T]) :- T: Sized, WF(T)
            builder.push_bound_ty(|builder, ty| {
                let sized = builder.db.well_known_trait_id(WellKnownTrait::Sized);
                builder.push_clause(
                    WellFormed::Ty(TyKind::Slice(ty.clone()).intern(builder.interner())),
                    sized
                        .map(|id| {
                            DomainGoal::Holds(WhereClause::Implemented(TraitRef {
                                trait_id: id,
                                substitution: Substitution::from1(interner, ty.clone()),
                            }))
                        })
                        .into_iter()
                        .chain(Some(DomainGoal::WellFormed(WellFormed::Ty(ty)))),
                );
            });
        }
        TyKind::Array(..) => {
            // forall<T. const N: usize> WF([T, N]) :- T: Sized
            let interner = builder.interner();
            let binders = Binders::new(
                VariableKinds::from_iter(
                    interner,
                    [
                        VariableKind::Ty(TyVariableKind::General),
                        VariableKind::Const(
                            TyKind::Scalar(Scalar::Uint(UintTy::Usize)).intern(interner),
                        ),
                    ],
                ),
                PhantomData::<I>,
            );
            builder.push_binders(binders, |builder, PhantomData| {
                let placeholders_in_scope = builder.placeholders_in_scope();
                let placeholder_count = placeholders_in_scope.len();
                let ty = placeholders_in_scope[placeholder_count - 2]
                    .assert_ty_ref(interner)
                    .clone();
                let size = placeholders_in_scope[placeholder_count - 1]
                    .assert_const_ref(interner)
                    .clone();

                let sized = builder.db.well_known_trait_id(WellKnownTrait::Sized);
                let array_ty = TyKind::Array(ty.clone(), size).intern(interner);
                builder.push_clause(
                    WellFormed::Ty(array_ty),
                    sized
                        .map(|id| {
                            DomainGoal::Holds(WhereClause::Implemented(TraitRef {
                                trait_id: id,
                                substitution: Substitution::from1(interner, ty.clone()),
                            }))
                        })
                        .into_iter()
                        .chain(Some(DomainGoal::WellFormed(WellFormed::Ty(ty)))),
                );
            });
        }
        TyKind::Tuple(len, _) => {
            // WF((T0, ..., Tn, U)) :- T0: Sized, ..., Tn: Sized, WF(T0), ..., WF(Tn), WF(U)
            let interner = builder.interner();
            let binders = Binders::new(
                VariableKinds::from_iter(
                    interner,
                    iter::repeat_with(|| VariableKind::Ty(TyVariableKind::General)).take(*len),
                ),
                PhantomData::<I>,
            );
            builder.push_binders(binders, |builder, PhantomData| {
                let placeholders_in_scope = builder.placeholders_in_scope();

                let substs = Substitution::from_iter(
                    builder.interner(),
                    &placeholders_in_scope[placeholders_in_scope.len() - len..],
                );

                let tuple_ty = TyKind::Tuple(*len, substs.clone()).intern(interner);
                let sized = builder.db.well_known_trait_id(WellKnownTrait::Sized);
                builder.push_clause(
                    WellFormed::Ty(tuple_ty),
                    substs.as_slice(interner)[..*len - 1]
                        .iter()
                        .filter_map(|s| {
                            let ty_var = s.assert_ty_ref(interner).clone();
                            sized.map(|id| {
                                DomainGoal::Holds(WhereClause::Implemented(TraitRef {
                                    trait_id: id,
                                    substitution: Substitution::from1(interner, ty_var),
                                }))
                            })
                        })
                        .chain(substs.iter(interner).map(|subst| {
                            DomainGoal::WellFormed(WellFormed::Ty(
                                subst.assert_ty_ref(interner).clone(),
                            ))
                        })),
                );
            });
        }
        TyKind::Closure(_, _) | TyKind::Coroutine(_, _) | TyKind::CoroutineWitness(_, _) => {
            let ty = generalize::Generalize::apply(builder.db.interner(), ty.clone());
            builder.push_binders(ty, |builder, ty| {
                builder.push_fact(WellFormed::Ty(ty));
            });
        }
        TyKind::Placeholder(_) => {
            builder.push_fact(WellFormed::Ty(ty.clone()));
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
            let ty = generalize::Generalize::apply(builder.db.interner(), ty.clone());
            builder.push_binders(ty, |builder, ty| builder.push_fact(WellFormed::Ty(ty)));
        }
        TyKind::BoundVar(_) => return Err(Floundered),
        TyKind::Dyn(dyn_ty) => {
            // FIXME(#203)
            // - Object safety? (not needed with RFC 2027)
            // - Implied bounds
            // - Bounds on the associated types
            // - Checking that all associated types are specified, including
            //   those on supertraits.
            // - For trait objects with GATs, if we allow them in the future,
            //   check that the bounds are fully general (
            //   `dyn for<'a> StreamingIterator<Item<'a> = &'a ()>` is OK,
            //   `dyn StreamingIterator<Item<'static> = &'static ()>` is not).
            let generalized_ty =
                generalize::Generalize::apply(builder.db.interner(), dyn_ty.clone());
            builder.push_binders(generalized_ty, |builder, dyn_ty| {
                let bounds = dyn_ty
                    .bounds
                    .substitute(interner, &[ty.clone().cast::<GenericArg<I>>(interner)]);

                let mut wf_goals = Vec::new();

                wf_goals.extend(bounds.iter(interner).flat_map(|bound| {
                    bound.map_ref(|bound| -> Vec<_> {
                        match bound {
                            WhereClause::Implemented(trait_ref) => {
                                vec![DomainGoal::WellFormed(WellFormed::Trait(trait_ref.clone()))]
                            }
                            WhereClause::AliasEq(_)
                            | WhereClause::LifetimeOutlives(_)
                            | WhereClause::TypeOutlives(_) => vec![],
                        }
                    })
                }));

                builder.push_clause(WellFormed::Ty(ty.clone()), wf_goals);
            });
        }
    }
    Ok(())
}

fn match_alias_ty<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    environment: &Environment<I>,
    alias: &AliasTy<I>,
) {
    if let AliasTy::Projection(projection_ty) = alias {
        builder
            .db
            .associated_ty_data(projection_ty.associated_ty_id)
            .to_program_clauses(builder, environment)
    }
}

#[instrument(level = "debug", skip(db))]
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
