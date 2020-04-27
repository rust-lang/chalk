use self::builder::ClauseBuilder;
use self::env_elaborator::elaborate_env_clauses;
use self::program_clauses::ToProgramClauses;
use crate::split::Split;
use crate::RustIrDatabase;
use chalk_engine::context::Floundered;
use chalk_ir::cast::Cast;
use chalk_ir::could_match::CouldMatch;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use rustc_hash::FxHashSet;

pub mod builder;
mod builtin_traits;
mod env_elaborator;
mod generalize;
pub mod program_clauses;

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
pub fn push_auto_trait_impls<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    auto_trait_id: TraitId<I>,
    struct_id: StructId<I>,
) {
    debug_heading!(
        "push_auto_trait_impls({:?}, {:?})",
        auto_trait_id,
        struct_id
    );

    let struct_datum = &builder.db.struct_datum(struct_id);
    let interner = builder.interner();

    // Must be an auto trait.
    assert!(builder.db.trait_datum(auto_trait_id).is_auto_trait());

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(
        builder.db.trait_datum(auto_trait_id).binders.len(interner),
        1
    );

    // If there is a `impl AutoTrait for Foo<..>` or `impl !AutoTrait
    // for Foo<..>`, where `Foo` is the struct we're looking at, then
    // we don't generate our own rules.
    if builder.db.impl_provided_for(auto_trait_id, struct_id) {
        debug!("impl provided");
        return;
    }

    let binders = struct_datum.binders.map_ref(|b| &b.fields);
    builder.push_binders(&binders, |builder, fields| {
        let self_ty: Ty<_> = ApplicationTy {
            name: struct_id.cast(interner),
            substitution: builder.substitution_in_scope(),
        }
        .intern(interner);

        // trait_ref = `MyStruct<...>: MyAutoTrait`
        let auto_trait_ref = TraitRef {
            trait_id: auto_trait_id,
            substitution: Substitution::from1(interner, self_ty),
        };

        // forall<P0..Pn> { // generic parameters from struct
        //   MyStruct<...>: MyAutoTrait :-
        //      Field0: MyAutoTrait,
        //      ...
        //      FieldN: MyAutoTrait
        // }
        builder.push_clause(
            auto_trait_ref,
            fields.iter().map(|field_ty| TraitRef {
                trait_id: auto_trait_id,
                substitution: Substitution::from1(interner, field_ty.clone()),
            }),
        );
    });
}

/// Given some goal `goal` that must be proven, along with
/// its `environment`, figures out the program clauses that apply
/// to this goal from the Rust program. So for example if the goal
/// is `Implemented(T: Clone)`, then this function might return clauses
/// derived from the trait `Clone` and its impls.
pub(crate) fn program_clauses_for_goal<'db, I: Interner>(
    db: &'db dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    goal: &DomainGoal<I>,
) -> Result<Vec<ProgramClause<I>>, Floundered> {
    debug_heading!(
        "program_clauses_for_goal(goal={:?}, environment={:?})",
        goal,
        environment
    );
    let interner = db.interner();

    // FIXME: change this to use `.chain().filter()`
    let mut vec = vec![];
    vec.extend(db.custom_clauses());
    program_clauses_that_could_match(db, environment, goal, &mut vec)?;
    vec.retain(|c| c.could_match(interner, goal));
    vec.extend(
        db.program_clauses_for_env(environment)
            .iter(interner)
            .filter(|c| (*c).could_match(interner, goal))
            .cloned(),
    );

    debug!("vec = {:#?}", vec);

    Ok(vec)
}

/// Returns a set of program clauses that could possibly match
/// `goal`. This can be any superset of the correct set, but the
/// more precise you can make it, the more efficient solving will
/// be.
fn program_clauses_that_could_match<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    goal: &DomainGoal<I>,
    clauses: &mut Vec<ProgramClause<I>>,
) -> Result<(), Floundered> {
    let interner = db.interner();
    let builder = &mut ClauseBuilder::new(db, clauses);

    debug_heading!("program_clauses_that_could_match(goal={:?})", goal);

    match goal {
        DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
            let trait_id = trait_ref.trait_id;

            let trait_datum = db.trait_datum(trait_id);

            if trait_datum.is_non_enumerable_trait() || trait_datum.is_auto_trait() {
                let self_ty = trait_ref.self_type_parameter(interner);
                if self_ty.bound(interner).is_some() || self_ty.inference_var(interner).is_some() {
                    return Err(Floundered);
                }
            }

            // This is needed for the coherence related impls, as well
            // as for the `Implemented(Foo) :- FromEnv(Foo)` rule.
            trait_datum.to_program_clauses(builder);

            for impl_id in db.impls_for_trait(
                trait_ref.trait_id,
                trait_ref.substitution.parameters(interner),
            ) {
                db.impl_datum(impl_id).to_program_clauses(builder);
            }

            // If this is a `Foo: Send` (or any auto-trait), then add
            // the automatic impls for `Foo`.
            let trait_datum = db.trait_datum(trait_id);
            if trait_datum.is_auto_trait() {
                match trait_ref.self_type_parameter(interner).data(interner) {
                    TyData::Apply(apply) => match &apply.name {
                        TypeName::Struct(struct_id) => {
                            push_auto_trait_impls(builder, trait_id, *struct_id);
                        }
                        _ => {}
                    },
                    TyData::InferenceVar(_) | TyData::BoundVar(_) => {
                        return Err(Floundered);
                    }
                    _ => {}
                }
            }

            // If the self type `S` is a `dyn trait` type, we wish to generate program-clauses
            // that indicates that it implements its own traits. For example, a `dyn Write` type
            // implements `Write` and so on.
            //
            // To see how this works, consider as an example the type `dyn Fn(&u8)`. This is
            // really shorthand for `dyn for<'a> Fn<(&'a u8), Output = ()>`, and we represent
            // that type as something like this:
            //
            // ```
            // dyn(exists<T> {
            //     forall<'a> { Implemented(T: Fn<'a>) },
            //     forall<'a> { AliasEq(<T as Fn<'a>>::Output, ()) },
            // })
            // ```
            //
            // so what we will do is to generate one program clause
            // for each of the conditions. Thus we get two program
            // clauses:
            //
            // ```
            // forall<'a> { Implemented(dyn Fn(&u8): Fn<(&'a u8)>) }
            // ```
            //
            // and
            //
            // ```
            // forall<'a> { AliasEq(<dyn Fn(&u8) as Fn<'a>>::Output, ()) },
            // ```
            //
            // FIXME. This is presently rather wasteful, in that we
            // don't check that the these program clauses we are
            // generating are actually relevant to the goal `goal`
            // that we are actually *trying* to prove (though there is
            // some later code that will screen out irrelevant
            // stuff).
            //
            // In other words, in our example, if we were trying to
            // prove `Implemented(dyn Fn(&u8): Clone)`, we would have
            // generated two clauses that are totally irrelevant to
            // that goal, because they let us prove other things but
            // not `Clone`.
            let self_ty = trait_ref.self_type_parameter(interner);
            if let TyData::Dyn(dyn_ty) = self_ty.data(interner) {
                // In this arm, `self_ty` is the `dyn Fn(&u8)`,
                // and `bounded_ty` is the `exists<T> { .. }`
                // clauses shown above.

                // Turn free BoundVars in the type into new existentials. E.g.
                // we might get some `dyn Foo<?X>`, and we don't want to return
                // a clause with a free variable. We can instead return a
                // slightly more general clause by basically turning this into
                // `exists<A> dyn Foo<A>`.
                let generalized_dyn_ty = generalize::Generalize::apply(db.interner(), dyn_ty);

                builder.push_binders(&generalized_dyn_ty, |builder, dyn_ty| {
                    for exists_qwc in dyn_ty.bounds.map_ref(|r| r.iter(interner)) {
                        // Replace the `T` from `exists<T> { .. }` with `self_ty`,
                        // yielding clases like
                        //
                        // ```
                        // forall<'a> { Implemented(dyn Fn(&u8): Fn<(&'a u8)>) }
                        // ```
                        let qwc =
                            exists_qwc.substitute(interner, &[self_ty.clone().cast(interner)]);

                        builder.push_binders(&qwc, |builder, wc| {
                            builder.push_fact(wc);
                        });
                    }
                });
            }

            match self_ty.data(interner) {
                TyData::Apply(ApplicationTy {
                    name: TypeName::OpaqueType(opaque_ty_id),
                    ..
                })
                | TyData::Alias(AliasTy::Opaque(OpaqueTy { opaque_ty_id, .. })) => {
                    db.opaque_ty_data(*opaque_ty_id).to_program_clauses(builder);
                }
                _ => {}
            }

            if let Some(well_known) = trait_datum.well_known {
                builtin_traits::add_builtin_program_clauses(
                    db,
                    builder,
                    well_known,
                    trait_ref,
                    self_ty.data(interner),
                );
            }
        }
        DomainGoal::Holds(WhereClause::AliasEq(alias_eq)) => match &alias_eq.alias {
            AliasTy::Projection(proj) => db
                .associated_ty_data(proj.associated_ty_id)
                .to_program_clauses(builder),
            AliasTy::Opaque(opaque_ty) => db
                .opaque_ty_data(opaque_ty.opaque_ty_id)
                .to_program_clauses(builder),
        },
        DomainGoal::WellFormed(WellFormed::Trait(trait_predicate)) => {
            db.trait_datum(trait_predicate.trait_id)
                .to_program_clauses(builder);
        }
        DomainGoal::WellFormed(WellFormed::Ty(ty))
        | DomainGoal::IsUpstream(ty)
        | DomainGoal::DownstreamType(ty) => match_ty(builder, environment, ty)?,
        DomainGoal::IsFullyVisible(ty) | DomainGoal::IsLocal(ty) => {
            match_ty(builder, environment, ty)?
        }
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

                // Flounder if the self-type is unknown and the trait is non-enumerable.
                //
                // e.g., Normalize(<?X as Iterator>::Item = u32)
                if (alias.self_type_parameter(interner).is_var(interner))
                    && trait_datum.is_non_enumerable_trait()
                {
                    return Err(Floundered);
                }

                push_program_clauses_for_associated_type_values_in_impls_of(
                    builder,
                    trait_id,
                    trait_parameters,
                );
            }
            AliasTy::Opaque(_) => (),
        },
        DomainGoal::LocalImplAllowed(trait_ref) => db
            .trait_datum(trait_ref.trait_id)
            .to_program_clauses(builder),
        DomainGoal::Compatible(()) => (),
        DomainGoal::Reveal(()) => (),
    };

    Ok(())
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
fn push_program_clauses_for_associated_type_values_in_impls_of<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    trait_id: TraitId<I>,
    trait_parameters: &[Parameter<I>],
) {
    debug_heading!(
        "push_program_clauses_for_associated_type_values_in_impls_of(\
         trait_id={:?}, \
         trait_parameters={:?})",
        trait_id,
        trait_parameters,
    );

    for impl_id in builder.db.impls_for_trait(trait_id, trait_parameters) {
        let impl_datum = builder.db.impl_datum(impl_id);
        if !impl_datum.is_positive() {
            continue;
        }

        debug!("impl_id = {:?}", impl_id);

        for &atv_id in &impl_datum.associated_ty_value_ids {
            let atv = builder.db.associated_ty_value(atv_id);
            debug!("atv_id = {:?} atv = {:#?}", atv_id, atv);
            atv.to_program_clauses(builder);
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
    Ok(match ty.data(interner) {
        TyData::Apply(application_ty) => match_type_name(builder, interner, application_ty),
        TyData::Placeholder(_) => {
            builder.push_clause(WellFormed::Ty(ty.clone()), Some(FromEnv::Ty(ty.clone())));
        }
        TyData::Alias(AliasTy::Projection(proj)) => builder
            .db
            .associated_ty_data(proj.associated_ty_id)
            .to_program_clauses(builder),
        TyData::Alias(AliasTy::Opaque(opaque_ty)) => builder
            .db
            .opaque_ty_data(opaque_ty.opaque_ty_id)
            .to_program_clauses(builder),
        TyData::Function(quantified_ty) => {
            builder.push_fact(WellFormed::Ty(ty.clone()));
            quantified_ty
                .substitution
                .iter(interner)
                .map(|p| p.assert_ty_ref(interner))
                .map(|ty| match_ty(builder, environment, &ty))
                .collect::<Result<_, Floundered>>()?;
        }
        TyData::BoundVar(_) | TyData::InferenceVar(_) => return Err(Floundered),
        TyData::Dyn(_) => {}
    })
}

fn match_type_name<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    interner: &I,
    application: &ApplicationTy<I>,
) {
    match application.name {
        TypeName::Struct(struct_id) => match_struct(builder, struct_id),
        TypeName::OpaqueType(opaque_ty_id) => builder
            .db
            .opaque_ty_data(opaque_ty_id)
            .to_program_clauses(builder),
        TypeName::Error => {}
        TypeName::AssociatedType(type_id) => builder
            .db
            .associated_ty_data(type_id)
            .to_program_clauses(builder),
        TypeName::Scalar(_) => {
            builder.push_fact(WellFormed::Ty(application.clone().intern(interner)))
        }
        TypeName::Tuple(_) => {
            builder.push_fact(WellFormed::Ty(application.clone().intern(interner)))
        }
    }
}

fn match_alias_ty<I: Interner>(builder: &mut ClauseBuilder<'_, I>, alias: &AliasTy<I>) {
    match alias {
        AliasTy::Projection(projection_ty) => builder
            .db
            .associated_ty_data(projection_ty.associated_ty_id)
            .to_program_clauses(builder),
        _ => (),
    }
}

fn match_struct<I: Interner>(builder: &mut ClauseBuilder<'_, I>, struct_id: StructId<I>) {
    builder
        .db
        .struct_datum(struct_id)
        .to_program_clauses(builder)
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
        elaborate_env_clauses(db, &last_round.drain().collect::<Vec<_>>(), &mut next_round);
        last_round.extend(
            next_round
                .drain()
                .filter(|clause| closure.insert(clause.clone())),
        );
    }

    ProgramClauses::from(db.interner(), closure)
}
