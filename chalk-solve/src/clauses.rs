use self::builder::ClauseBuilder;
use self::env_elaborator::elaborate_env_clauses;
use self::program_clauses::ToProgramClauses;
use crate::RustIrDatabase;
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::family::ChalkIr;
use chalk_ir::*;
use rustc_hash::FxHashSet;

mod builder;
mod env_elaborator;
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
pub fn push_auto_trait_impls(
    auto_trait_id: TraitId,
    struct_id: StructId,
    db: &dyn RustIrDatabase,
    clauses: &mut Vec<ProgramClause<ChalkIr>>,
) {
    debug_heading!(
        "push_auto_trait_impls({:?}, {:?})",
        auto_trait_id,
        struct_id
    );

    let mut builder = ClauseBuilder::new(db, clauses);

    let struct_datum = &builder.db.struct_datum(struct_id);

    // Must be an auto trait.
    assert!(builder.db.trait_datum(auto_trait_id).is_auto_trait());

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(builder.db.trait_datum(auto_trait_id).binders.len(), 1);

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
            name: struct_id.cast(),
            parameters: builder.placeholders_in_scope().to_vec(),
        }
        .cast();

        // trait_ref = `MyStruct<...>: MyAutoTrait`
        let auto_trait_ref = TraitRef {
            trait_id: auto_trait_id,
            parameters: vec![self_ty.cast()],
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
                parameters: vec![field_ty.clone().cast()],
            }),
        );
    });
}

/// Given some goal `goal` that must be proven, along with
/// its `environment`, figures out the program clauses that apply
/// to this goal from the Rust program. So for example if the goal
/// is `Implemented(T: Clone)`, then this function might return clauses
/// derived from the trait `Clone` and its impls.
pub(crate) fn program_clauses_for_goal<'db>(
    db: &'db dyn RustIrDatabase,
    environment: &Environment<ChalkIr>,
    goal: &DomainGoal<ChalkIr>,
) -> Vec<ProgramClause<ChalkIr>> {
    debug_heading!(
        "program_clauses_for_goal(goal={:?}, environment={:?})",
        goal,
        environment
    );

    let mut vec = vec![];
    vec.extend(db.custom_clauses());
    program_clauses_that_could_match(db, environment, goal, &mut vec);
    program_clauses_for_env(db, environment, &mut vec);
    vec.retain(|c| c.could_match(goal));

    debug!("vec = {:#?}", vec);

    vec
}

/// Returns a set of program clauses that could possibly match
/// `goal`. This can be any superset of the correct set, but the
/// more precise you can make it, the more efficient solving will
/// be.
fn program_clauses_that_could_match(
    db: &dyn RustIrDatabase,
    environment: &Environment<ChalkIr>,
    goal: &DomainGoal<ChalkIr>,
    clauses: &mut Vec<ProgramClause<ChalkIr>>,
) {
    match goal {
        DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
            let trait_id = trait_ref.trait_id;

            // This is needed for the coherence related impls, as well
            // as for the `Implemented(Foo) :- FromEnv(Foo)` rule.
            db.trait_datum(trait_id).to_program_clauses(db, clauses);

            for impl_id in db.impls_for_trait(trait_ref.trait_id, &trait_ref.parameters) {
                db.impl_datum(impl_id).to_program_clauses(db, clauses);
            }

            // If this is a `Foo: Send` (or any auto-trait), then add
            // the automatic impls for `Foo`.
            let trait_datum = db.trait_datum(trait_id);
            if trait_datum.is_auto_trait() {
                match trait_ref.parameters[0].assert_ty_ref() {
                    Ty::Apply(apply) => {
                        if let TypeName::TypeKindId(TypeKindId::StructId(struct_id)) = apply.name {
                            push_auto_trait_impls(trait_id, struct_id, db, clauses);
                        }
                    }
                    Ty::InferenceVar(_) => {
                        panic!("auto-traits should flounder if nothing is known")
                    }
                    _ => {}
                }
            }

            // If the self type is `dyn Foo` (or `impl Foo`), then we generate clauses like:
            //
            // ```notrust
            // Implemented(dyn Foo: Foo)
            // ```
            //
            // FIXME. This is presently rather wasteful, in that we
            // don't check that the `dyn Foo: Foo` trait is relevant
            // to the goal `goal` that we are actually *trying* to
            // prove (though there is some later code that will screen
            // out irrelevant stuff). In other words, we might be
            // trying to prove `dyn Foo: Bar`, in which case the clause
            // for `dyn Foo: Foo` is not particularly relevant.
            match trait_ref.self_type_parameter() {
                Some(Ty::Opaque(qwc)) | Some(Ty::Dyn(qwc)) => {
                    let self_ty = trait_ref.self_type_parameter().unwrap(); // This cannot be None
                    let wc = qwc.substitute(&[self_ty.cast()]);
                    clauses.extend(wc.into_iter().casted());
                }
                _ => {}
            }

            // TODO sized, unsize_trait, builtin impls?
        }
        DomainGoal::Holds(WhereClause::ProjectionEq(projection_predicate)) => {
            db.associated_ty_data(projection_predicate.projection.associated_ty_id)
                .to_program_clauses(db, clauses);
        }
        DomainGoal::WellFormed(WellFormed::Trait(trait_predicate)) => {
            db.trait_datum(trait_predicate.trait_id)
                .to_program_clauses(db, clauses);
        }
        DomainGoal::WellFormed(WellFormed::Ty(ty))
        | DomainGoal::IsUpstream(ty)
        | DomainGoal::DownstreamType(ty) => match_ty(db, environment, ty, clauses),
        DomainGoal::IsFullyVisible(ty) | DomainGoal::IsLocal(ty) => {
            match_ty(db, environment, ty, clauses)
        }
        DomainGoal::FromEnv(_) => (), // Computed in the environment
        DomainGoal::Normalize(Normalize { projection, ty: _ }) => {
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
            let associated_ty_datum = db.associated_ty_data(projection.associated_ty_id);
            let trait_id = associated_ty_datum.trait_id;
            let (_, trait_parameters, _) = db.split_projection(projection);
            push_program_clauses_for_associated_type_values_in_impls_of(
                db,
                trait_id,
                trait_parameters,
                clauses,
            );
        }
        DomainGoal::LocalImplAllowed(trait_ref) => db
            .trait_datum(trait_ref.trait_id)
            .to_program_clauses(db, clauses),
        DomainGoal::Compatible(()) => (),
    };
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
fn push_program_clauses_for_associated_type_values_in_impls_of(
    db: &dyn RustIrDatabase,
    trait_id: TraitId,
    trait_parameters: &[Parameter<ChalkIr>],
    clauses: &mut Vec<ProgramClause<ChalkIr>>,
) {
    for impl_id in db.impls_for_trait(trait_id, trait_parameters) {
        let impl_datum = db.impl_datum(impl_id);
        if !impl_datum.is_positive() {
            continue;
        }

        for atv in &impl_datum.binders.value.associated_ty_values {
            atv.to_program_clauses(db, clauses);
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
fn match_ty(
    db: &dyn RustIrDatabase,
    environment: &Environment<ChalkIr>,
    ty: &Ty<ChalkIr>,
    clauses: &mut Vec<ProgramClause<ChalkIr>>,
) {
    match ty {
        Ty::Apply(application_ty) => match application_ty.name {
            TypeName::TypeKindId(type_kind_id) => match_type_kind(db, type_kind_id, clauses),
            TypeName::Placeholder(_) | TypeName::Error => {}
            TypeName::AssociatedType(type_id) => db
                .associated_ty_data(type_id)
                .to_program_clauses(db, clauses),
        },
        Ty::Projection(projection_ty) => db
            .associated_ty_data(projection_ty.associated_ty_id)
            .to_program_clauses(db, clauses),
        Ty::ForAll(quantified_ty) => match_ty(db, environment, &quantified_ty.ty, clauses),
        Ty::BoundVar(_) => {}
        Ty::InferenceVar(_) => panic!("should have floundered"),
        Ty::Dyn(_) | Ty::Opaque(_) => {}
    }
}

fn match_type_kind(
    db: &dyn RustIrDatabase,
    type_kind_id: TypeKindId,
    clauses: &mut Vec<ProgramClause<ChalkIr>>,
) {
    match type_kind_id {
        TypeKindId::TypeId(type_id) => db
            .associated_ty_data(type_id)
            .to_program_clauses(db, clauses),
        TypeKindId::TraitId(trait_id) => db.trait_datum(trait_id).to_program_clauses(db, clauses),
        TypeKindId::StructId(struct_id) => {
            db.struct_datum(struct_id).to_program_clauses(db, clauses)
        }
    }
}

fn program_clauses_for_env<'db>(
    db: &'db dyn RustIrDatabase,
    environment: &Environment<ChalkIr>,
    clauses: &mut Vec<ProgramClause<ChalkIr>>,
) {
    let mut last_round = FxHashSet::default();
    elaborate_env_clauses(db, &environment.clauses, &mut last_round);

    let mut closure = last_round.clone();
    let mut next_round = FxHashSet::default();
    while !last_round.is_empty() {
        elaborate_env_clauses(db, &last_round.drain().collect(), &mut next_round);
        last_round.extend(
            next_round
                .drain()
                .filter(|clause| closure.insert(clause.clone())),
        );
    }

    clauses.extend(closure.drain())
}
