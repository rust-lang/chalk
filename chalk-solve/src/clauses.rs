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

pub mod builder;
mod builtin_traits;
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
    let interner = builder.interner();
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

// TODO add Floundered error instead of using Option

/// Given some goal `goal` that must be proven, along with
/// its `environment`, figures out the program clauses that apply
/// to this goal from the Rust program. So for example if the goal
/// is `Implemented(T: Clone)`, then this function might return clauses
/// derived from the trait `Clone` and its impls.
pub(crate) fn program_clauses_for_goal<'db, I: Interner>(
    db: &'db dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    goal: &DomainGoal<I>,
) -> Option<Vec<ProgramClause<I>>> {
    debug_heading!(
        "program_clauses_for_goal(goal={:?}, environment={:?})",
        goal,
        environment
    );
    let interner = db.interner();

    let mut vec = vec![];
    vec.extend(db.custom_clauses());
    program_clauses_that_could_match(db, environment, goal, &mut vec)?;
    program_clauses_for_env(db, environment, &mut vec);
    vec.retain(|c| c.could_match(interner, goal));

    debug!("vec = {:#?}", vec);

    Some(vec)
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
) -> Option<()> {
    let interner = db.interner();
    let builder = &mut ClauseBuilder::new(db, clauses);

    match goal {
        DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
            let trait_id = trait_ref.trait_id;

            let trait_datum = db.trait_datum(trait_id);

            if trait_datum.is_non_enumerable_trait() || trait_datum.is_auto_trait() {
                let self_ty = trait_ref.self_type_parameter(interner);
                if self_ty.bound(interner).is_some() || self_ty.inference_var(interner).is_some() {
                    return None;
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
                    TyData::Apply(apply) => {
                        if let Some(struct_id) = db.as_struct_id(&apply.name) {
                            push_auto_trait_impls(builder, trait_id, struct_id);
                        }
                    }
                    TyData::InferenceVar(_) | TyData::BoundVar(_) => {
                        return None;
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
        DomainGoal::Holds(WhereClause::AliasEq(alias_predicate)) => {
            db.associated_ty_data(alias_predicate.alias.associated_ty_id)
                .to_program_clauses(builder);
        }
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
        DomainGoal::Normalize(Normalize { alias, ty: _ }) => {
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
            let associated_ty_datum = db.associated_ty_data(alias.associated_ty_id);
            let trait_id = associated_ty_datum.trait_id;
            let trait_parameters = db.trait_parameters_from_projection(alias);

            push_program_clauses_for_associated_type_values_in_impls_of(
                builder,
                trait_id,
                trait_parameters,
            );
        }
        DomainGoal::LocalImplAllowed(trait_ref) => db
            .trait_datum(trait_ref.trait_id)
            .to_program_clauses(builder),
        DomainGoal::Compatible(()) => (),
    };

    Some(())
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
) -> Option<()> {
    let interner = builder.interner();
    Some(match ty.data(interner) {
        TyData::Apply(application_ty) => match_type_name(builder, application_ty.name),
        TyData::Placeholder(_) => {
            builder.push_clause(WellFormed::Ty(ty.clone()), Some(FromEnv::Ty(ty.clone())));
        }
        TyData::Alias(alias_ty) => builder
            .db
            .associated_ty_data(alias_ty.associated_ty_id)
            .to_program_clauses(builder),
        TyData::Function(quantified_ty) => {
            builder.push_fact(WellFormed::Ty(ty.clone()));
            quantified_ty
                .substitution
                .iter(interner)
                .map(|p| p.assert_ty_ref(interner))
                .map(|ty| match_ty(builder, environment, &ty))
                .collect::<Option<_>>()?;
        }
        TyData::BoundVar(_) | TyData::InferenceVar(_) => return None,
        TyData::Dyn(_) => {}
    })
}

fn match_type_name<I: Interner>(builder: &mut ClauseBuilder<'_, I>, name: TypeName<I>) {
    match name {
        TypeName::Struct(struct_id) => match_struct(builder, struct_id),
        TypeName::Error => {}
        TypeName::AssociatedType(type_id) => builder
            .db
            .associated_ty_data(type_id)
            .to_program_clauses(builder),
    }
}

fn match_struct<I: Interner>(builder: &mut ClauseBuilder<'_, I>, struct_id: StructId<I>) {
    builder
        .db
        .struct_datum(struct_id)
        .to_program_clauses(builder)
}

fn program_clauses_for_env<'db, I: Interner>(
    db: &'db dyn RustIrDatabase<I>,
    environment: &Environment<I>,
    clauses: &mut Vec<ProgramClause<I>>,
) {
    clauses.extend(environment.clauses.iter(db.interner()).cloned());

    let mut last_round = FxHashSet::default();
    elaborate_env_clauses(
        db,
        environment.clauses.as_slice(db.interner()),
        &mut last_round,
    );

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

    clauses.extend(closure.drain())
}

mod generalize {
    use chalk_engine::fallible::Fallible;
    use chalk_ir::{
        fold::{Fold, Folder},
        interner::Interner,
        Binders, BoundVar, DebruijnIndex, Lifetime, LifetimeData, ParameterKind, Ty, TyData,
    };
    use std::collections::HashMap;

    pub struct Generalize<'i, I: Interner> {
        binders: Vec<ParameterKind<()>>,
        mapping: HashMap<BoundVar, usize>,
        interner: &'i I,
    }

    impl<I: Interner> Generalize<'_, I> {
        pub fn apply<T: Fold<I, I>>(interner: &I, value: &T) -> Binders<T::Result> {
            let mut generalize = Generalize {
                binders: Vec::new(),
                mapping: HashMap::new(),
                interner,
            };
            let value = value
                .fold_with(&mut generalize, DebruijnIndex::INNERMOST)
                .unwrap();
            Binders {
                binders: generalize.binders,
                value,
            }
        }
    }

    impl<'i, I: Interner> Folder<'i, I> for Generalize<'i, I> {
        fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
            self
        }

        fn fold_free_var_ty(
            &mut self,
            bound_var: BoundVar,
            outer_binder: DebruijnIndex,
        ) -> Fallible<Ty<I>> {
            let binder_vec = &mut self.binders;
            let new_index = self.mapping.entry(bound_var).or_insert_with(|| {
                let i = binder_vec.len();
                binder_vec.push(ParameterKind::Ty(()));
                i
            });
            let new_var = BoundVar::new(outer_binder, *new_index);
            Ok(TyData::BoundVar(new_var).intern(self.interner()))
        }

        fn fold_free_var_lifetime(
            &mut self,
            bound_var: BoundVar,
            outer_binder: DebruijnIndex,
        ) -> Fallible<Lifetime<I>> {
            let binder_vec = &mut self.binders;
            let new_index = self.mapping.entry(bound_var).or_insert_with(|| {
                let i = binder_vec.len();
                binder_vec.push(ParameterKind::Ty(()));
                i
            });
            let new_var = BoundVar::new(outer_binder, *new_index);
            Ok(LifetimeData::BoundVar(new_var).intern(self.interner()))
        }

        fn interner(&self) -> &'i I {
            self.interner
        }

        fn target_interner(&self) -> &'i I {
            self.interner
        }
    }
}
