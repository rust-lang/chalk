use self::builder::ClauseBuilder;
use self::env_elaborator::elaborate_env_clauses;
use self::program_clauses::ToProgramClauses;
use crate::split::Split;
use crate::RustIrDatabase;
use chalk_ir::cast::Cast;
use chalk_ir::could_match::CouldMatch;
use chalk_ir::interner::Interner;
use chalk_ir::*;
use rustc_hash::FxHashSet;
use tracing::{debug, instrument};

pub mod builder;
mod builtin_traits;
mod dyn_ty;
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
#[instrument(level = "debug", skip(builder))]
pub fn push_auto_trait_impls<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    auto_trait_id: TraitId<I>,
    adt_id: AdtId<I>,
) {
    let adt_datum = &builder.db.adt_datum(adt_id);
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
    if builder.db.impl_provided_for(auto_trait_id, adt_id) {
        debug!("impl provided");
        return;
    }

    let binders = adt_datum.binders.map_ref(|b| &b.fields);
    builder.push_binders(&binders, |builder, fields| {
        let self_ty: Ty<_> = ApplicationTy {
            name: adt_id.cast(interner),
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
        let self_ty: Ty<_> = ApplicationTy {
            name: opaque_id.cast(interner),
            substitution: builder.substitution_in_scope(),
        }
        .intern(interner);

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

/// Given some goal `goal` that must be proven, along with
/// its `environment`, figures out the program clauses that apply
/// to this goal from the Rust program. So for example if the goal
/// is `Implemented(T: Clone)`, then this function might return clauses
/// derived from the trait `Clone` and its impls.
#[instrument(level = "debug", skip(db))]
pub(crate) fn program_clauses_for_goal<'db, I: Interner>(
    db: &'db dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    goal: &DomainGoal<I>,
) -> Result<Vec<ProgramClause<I>>, Floundered> {
    let interner = db.interner();

    let custom_clauses = db.custom_clauses().into_iter();
    let clauses_that_could_match =
        program_clauses_that_could_match(db, environment, goal).map(|cl| cl.into_iter())?;

    let clauses: Vec<ProgramClause<I>> = custom_clauses
        .chain(clauses_that_could_match)
        .chain(
            db.program_clauses_for_env(environment)
                .iter(interner)
                .cloned(),
        )
        .filter(|c| c.could_match(interner, goal))
        .collect();

    debug!("vec = {:#?}", clauses);

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

                if let TyData::Alias(AliasTy::Opaque(opaque_ty)) = self_ty.data(interner) {
                    if trait_datum.is_auto_trait() {
                        push_auto_trait_impls_opaque(builder, trait_id, opaque_ty.opaque_ty_id)
                    }
                } else if self_ty.bound_var(interner).is_some()
                    || self_ty.inference_var(interner).is_some()
                {
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
                        TypeName::Adt(adt_id) => {
                            push_auto_trait_impls(builder, trait_id, *adt_id);
                        }
                        _ => {}
                    },
                    TyData::InferenceVar(_, _) | TyData::BoundVar(_) => {
                        return Err(Floundered);
                    }
                    _ => {}
                }
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
            if let TyData::Dyn(_) = self_ty.data(interner) {
                dyn_ty::build_dyn_self_ty_clauses(db, builder, self_ty.clone())
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
                builtin_traits::add_builtin_program_clauses(db, builder, well_known, trait_ref)?;
            }
        }
        DomainGoal::Holds(WhereClause::AliasEq(alias_eq)) => match &alias_eq.alias {
            AliasTy::Projection(proj) => {
                let trait_self_ty = db
                    .trait_ref_from_projection(proj)
                    .self_type_parameter(interner);

                match trait_self_ty.data(interner) {
                    TyData::Apply(ApplicationTy {
                        name: TypeName::OpaqueType(opaque_ty_id),
                        ..
                    })
                    | TyData::Alias(AliasTy::Opaque(OpaqueTy { opaque_ty_id, .. })) => {
                        db.opaque_ty_data(*opaque_ty_id).to_program_clauses(builder);
                    }
                    _ => {}
                }

                db.associated_ty_data(proj.associated_ty_id)
                    .to_program_clauses(builder)
            }
            AliasTy::Opaque(opaque_ty) => db
                .opaque_ty_data(opaque_ty.opaque_ty_id)
                .to_program_clauses(builder),
        },
        DomainGoal::Holds(WhereClause::LifetimeOutlives(_)) => {}
        DomainGoal::WellFormed(WellFormed::Trait(trait_ref))
        | DomainGoal::LocalImplAllowed(trait_ref) => {
            db.trait_datum(trait_ref.trait_id)
                .to_program_clauses(builder);
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
                if (self_ty.is_var(interner)) && trait_datum.is_non_enumerable_trait() {
                    return Err(Floundered);
                }

                if let Some(well_known) = trait_datum.well_known {
                    builtin_traits::add_builtin_assoc_program_clauses(
                        db, builder, well_known, self_ty,
                    )?;
                }

                push_program_clauses_for_associated_type_values_in_impls_of(
                    builder,
                    trait_id,
                    trait_parameters,
                );
            }
            AliasTy::Opaque(_) => (),
        },
        DomainGoal::Compatible(()) | DomainGoal::Reveal(()) => (),
    };

    Ok(clauses)
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
    trait_id: TraitId<I>,
    trait_parameters: &[GenericArg<I>],
) {
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
        TyData::BoundVar(_) | TyData::InferenceVar(_, _) => return Err(Floundered),
        TyData::Dyn(_) => {}
    })
}

/// Lower a Rust IR application type to logic
fn match_type_name<I: Interner>(
    builder: &mut ClauseBuilder<'_, I>,
    interner: &I,
    application: &ApplicationTy<I>,
) {
    match application.name {
        TypeName::Adt(adt_id) => match_adt(builder, adt_id),
        TypeName::OpaqueType(opaque_ty_id) => builder
            .db
            .opaque_ty_data(opaque_ty_id)
            .to_program_clauses(builder),
        TypeName::Error => {}
        TypeName::AssociatedType(type_id) => builder
            .db
            .associated_ty_data(type_id)
            .to_program_clauses(builder),
        TypeName::FnDef(fn_def_id) => builder
            .db
            .fn_def_datum(fn_def_id)
            .to_program_clauses(builder),
        TypeName::Tuple(_)
        | TypeName::Scalar(_)
        | TypeName::Str
        | TypeName::Slice
        | TypeName::Raw(_)
        | TypeName::Ref(_)
        | TypeName::Array
        | TypeName::Never
        | TypeName::Closure(_) => {
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

fn match_adt<I: Interner>(builder: &mut ClauseBuilder<'_, I>, adt_id: AdtId<I>) {
    builder.db.adt_datum(adt_id).to_program_clauses(builder)
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
