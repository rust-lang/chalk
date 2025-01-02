use std::collections::HashSet;
use std::iter;
use std::ops::ControlFlow;

use crate::clauses::super_traits::super_traits;
use crate::clauses::ClauseBuilder;
use crate::rust_ir::AdtKind;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::{
    cast::Cast,
    interner::HasInterner,
    visit::{TypeSuperVisitable, TypeVisitable, TypeVisitor},
    Binders, Const, ConstValue, DebruijnIndex, DomainGoal, DynTy, EqGoal, Goal, LifetimeOutlives,
    QuantifiedWhereClauses, Substitution, TraitId, Ty, TyKind, TypeOutlives, WhereClause,
};

struct UnsizeParameterCollector<I: Interner> {
    interner: I,
    // FIXME should probably use a bitset instead
    parameters: HashSet<usize>,
}

impl<I: Interner> TypeVisitor<I> for UnsizeParameterCollector<I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn TypeVisitor<I, BreakTy = Self::BreakTy> {
        self
    }

    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> ControlFlow<()> {
        let interner = self.interner;

        match ty.kind(interner) {
            TyKind::BoundVar(bound_var) => {
                // check if bound var refers to the outermost binder
                if bound_var.debruijn.shifted_in() == outer_binder {
                    self.parameters.insert(bound_var.index);
                }
                ControlFlow::Continue(())
            }
            _ => ty.super_visit_with(self, outer_binder),
        }
    }

    fn visit_const(&mut self, constant: &Const<I>, outer_binder: DebruijnIndex) -> ControlFlow<()> {
        let interner = self.interner;

        if let ConstValue::BoundVar(bound_var) = constant.data(interner).value {
            // check if bound var refers to the outermost binder
            if bound_var.debruijn.shifted_in() == outer_binder {
                self.parameters.insert(bound_var.index);
            }
        }
        ControlFlow::Continue(())
    }

    fn interner(&self) -> I {
        self.interner
    }
}

fn outer_binder_parameters_used<I: Interner>(
    interner: I,
    v: &Binders<impl TypeVisitable<I> + HasInterner>,
) -> HashSet<usize> {
    let mut visitor = UnsizeParameterCollector {
        interner,
        parameters: HashSet::new(),
    };
    v.visit_with(&mut visitor, DebruijnIndex::INNERMOST);
    visitor.parameters
}

// has nothing to do with occurs check
struct ParameterOccurenceCheck<'p, I: Interner> {
    interner: I,
    parameters: &'p HashSet<usize>,
}

impl<'p, I: Interner> TypeVisitor<I> for ParameterOccurenceCheck<'p, I> {
    type BreakTy = ();

    fn as_dyn(&mut self) -> &mut dyn TypeVisitor<I, BreakTy = Self::BreakTy> {
        self
    }

    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> ControlFlow<()> {
        let interner = self.interner;

        match ty.kind(interner) {
            TyKind::BoundVar(bound_var) => {
                if bound_var.debruijn.shifted_in() == outer_binder
                    && self.parameters.contains(&bound_var.index)
                {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
            _ => ty.super_visit_with(self, outer_binder),
        }
    }

    fn visit_const(&mut self, constant: &Const<I>, outer_binder: DebruijnIndex) -> ControlFlow<()> {
        let interner = self.interner;

        match constant.data(interner).value {
            ConstValue::BoundVar(bound_var) => {
                if bound_var.debruijn.shifted_in() == outer_binder
                    && self.parameters.contains(&bound_var.index)
                {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
            _ => ControlFlow::Continue(()),
        }
    }

    fn interner(&self) -> I {
        self.interner
    }
}

fn uses_outer_binder_params<I: Interner>(
    interner: I,
    v: &Binders<impl TypeVisitable<I> + HasInterner>,
    parameters: &HashSet<usize>,
) -> bool {
    let mut visitor = ParameterOccurenceCheck {
        interner,
        parameters,
    };

    let flow = v.visit_with(&mut visitor, DebruijnIndex::INNERMOST);
    matches!(flow, ControlFlow::Break(_))
}

fn principal_trait_ref<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    bounds: &Binders<QuantifiedWhereClauses<I>>,
) -> Option<Binders<Binders<TraitRef<I>>>> {
    bounds
        .map_ref(|b| b.iter(db.interner()))
        .into_iter()
        .find_map(|b| {
            b.filter_map(|qwc| {
                qwc.as_ref().filter_map(|wc| match wc {
                    WhereClause::Implemented(trait_ref) => {
                        if db.trait_datum(trait_ref.trait_id).is_auto_trait() {
                            None
                        } else {
                            Some(trait_ref.clone())
                        }
                    }
                    _ => None,
                })
            })
        })
}

fn auto_trait_ids<'a, I: Interner>(
    db: &'a dyn RustIrDatabase<I>,
    bounds: &'a Binders<QuantifiedWhereClauses<I>>,
) -> impl Iterator<Item = TraitId<I>> + 'a {
    let interner = db.interner();

    bounds
        .skip_binders()
        .iter(interner)
        .filter_map(|clause| clause.trait_id())
        .filter(move |&id| db.trait_datum(id).is_auto_trait())
}

pub fn add_unsize_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: TraitRef<I>,
    _ty: TyKind<I>,
) {
    let interner = db.interner();

    let source_ty = trait_ref.self_type_parameter(interner);
    let target_ty = trait_ref
        .substitution
        .at(interner, 1)
        .assert_ty_ref(interner)
        .clone();

    let unsize_trait_id = trait_ref.trait_id;

    // N.B. here rustc asserts that `TraitRef` is not a higher-ranked bound
    // i.e. `for<'a> &'a T: Unsize<dyn Trait+'a>` is never provable.
    //
    // In chalk it would be awkward to implement and I am not sure
    // there is a need for it, the original comment states that this restriction
    // could be lifted.
    //
    // for more info visit `fn assemble_candidates_for_unsizing` and
    // `fn confirm_builtin_unsize_candidate` in rustc.

    match (source_ty.kind(interner), target_ty.kind(interner)) {
        // dyn TraitA + AutoA + 'a -> dyn TraitB + AutoB + 'b
        (
            TyKind::Dyn(DynTy {
                bounds: bounds_a,
                lifetime: lifetime_a,
            }),
            TyKind::Dyn(DynTy {
                bounds: bounds_b,
                lifetime: lifetime_b,
            }),
        ) => {
            let principal_trait_ref_a = principal_trait_ref(db, bounds_a);
            let principal_a = principal_trait_ref_a
                .as_ref()
                .map(|trait_ref| trait_ref.skip_binders().skip_binders().trait_id);
            let principal_b = principal_trait_ref(db, bounds_b)
                .map(|trait_ref| trait_ref.skip_binders().skip_binders().trait_id);

            // Include super traits in a list of auto traits for A,
            // to allow `dyn Trait -> dyn Trait + X` if `Trait: X`.
            let auto_trait_ids_a: Vec<_> = auto_trait_ids(db, bounds_a)
                .chain(principal_a.into_iter().flat_map(|principal_a| {
                    super_traits(db, principal_a)
                        .into_value_and_skipped_binders()
                        .0
                         .0
                        .into_iter()
                        .map(|x| x.skip_binders().trait_id)
                        .filter(|&x| db.trait_datum(x).is_auto_trait())
                }))
                .collect();

            let auto_trait_ids_b: Vec<_> = auto_trait_ids(db, bounds_b).collect();

            // If B has a principal, then A must as well
            // (i.e. we allow dropping principal, but not creating a principal out of thin air).
            // `AutoB` must be a subset of `AutoA`.
            let may_apply = principal_a.is_some() >= principal_b.is_some()
                && auto_trait_ids_b
                    .iter()
                    .all(|id_b| auto_trait_ids_a.iter().any(|id_a| id_a == id_b));

            if !may_apply {
                return;
            }

            // Check that source lifetime outlives target lifetime
            let lifetime_outlives_goal: Goal<I> = WhereClause::LifetimeOutlives(LifetimeOutlives {
                a: lifetime_a.clone(),
                b: lifetime_b.clone(),
            })
            .cast(interner);

            // COMMENT FROM RUSTC:
            // ------------------
            // Require that the traits involved in this upcast are **equal**;
            // only the **lifetime bound** is changed.
            //
            // This condition is arguably too strong -- it would
            // suffice for the source trait to be a *subtype* of the target
            // trait. In particular, changing from something like
            // `for<'a, 'b> Foo<'a, 'b>` to `for<'a> Foo<'a, 'a>` should be
            // permitted.
            // <...>
            // I've modified this to `.eq` because I want to continue rejecting
            // that [`old-lub-glb-object.rs`] test (as we have
            // done for quite some time) before we are firmly comfortable
            // with what our behavior should be there. -nikomatsakis
            // ------------------

            if principal_a == principal_b || principal_b.is_none() {
                // Construct a new trait object type by taking the source ty,
                // replacing auto traits of source with those of target,
                // and changing source lifetime to target lifetime.
                //
                // In order for the coercion to be valid, this new type
                // should be equal to target type.
                let new_source_ty = TyKind::Dyn(DynTy {
                    bounds: bounds_a.map_ref(|bounds| {
                        QuantifiedWhereClauses::from_iter(
                            interner,
                            bounds
                                .iter(interner)
                                .cloned()
                                .filter_map(|bound| {
                                    let Some(trait_id) = bound.trait_id() else {
                                        // Keep non-"implements" bounds as-is
                                        return Some(bound);
                                    };

                                    // Auto traits are already checked above, ignore them
                                    // (we'll use the ones from B below)
                                    if db.trait_datum(trait_id).is_auto_trait() {
                                        return None;
                                    }

                                    // The only "implements" bound that is not an auto trait, is the principal
                                    assert_eq!(Some(trait_id), principal_a);

                                    // Only include principal_a if the principal_b is also present
                                    // (this allows dropping principal, `dyn Tr+A -> dyn A`)
                                    principal_b.is_some().then(|| bound)
                                })
                                // Add auto traits from B (again, they are already checked above).
                                .chain(bounds_b.skip_binders().iter(interner).cloned().filter(
                                    |bound| {
                                        bound.trait_id().is_some_and(|trait_id| {
                                            db.trait_datum(trait_id).is_auto_trait()
                                        })
                                    },
                                )),
                        )
                    }),
                    lifetime: lifetime_b.clone(),
                })
                .intern(interner);

                // Check that new source is equal to target
                let eq_goal = EqGoal {
                    a: new_source_ty.cast(interner),
                    b: target_ty.clone().cast(interner),
                }
                .cast(interner);

                builder.push_clause(trait_ref, [eq_goal, lifetime_outlives_goal].iter());
            } else {
                // Conditions above imply that both of these are always `Some`
                // (b != None, b is Some iff a is Some).
                let principal_a = principal_a.unwrap();
                let principal_b = principal_b.unwrap();

                let principal_trait_ref_a = principal_trait_ref_a.unwrap();
                let applicable_super_traits = super_traits(db, principal_a)
                    .map(|(super_trait_refs, _)| super_trait_refs)
                    .into_iter()
                    .filter(|trait_ref| {
                        trait_ref.skip_binders().skip_binders().trait_id == principal_b
                    });

                for super_trait_ref in applicable_super_traits {
                    // `super_trait_ref` is, at this point, quantified over generic params of
                    // `principal_a` and relevant higher-ranked lifetimes that come from super
                    // trait elaboration (see comments on `super_traits()`).
                    //
                    // So if we have `trait Trait<'a, T>: for<'b> Super<'a, 'b, T> {}`,
                    // `super_trait_ref` can be something like
                    // `for<Self, 'a, T> for<'b> Self: Super<'a, 'b, T>`.
                    //
                    // We need to convert it into a bound for `DynTy`. We do this by substituting
                    // bound vars of `principal_trait_ref_a` and then fusing inner binders for
                    // higher-ranked lifetimes.
                    let rebound_super_trait_ref = principal_trait_ref_a.map_ref(|q_trait_ref_a| {
                        q_trait_ref_a
                            .map_ref(|trait_ref_a| {
                                super_trait_ref.substitute(interner, &trait_ref_a.substitution)
                            })
                            .fuse_binders(interner)
                    });

                    // Skip `for<Self>` binder. We'll rebind it immediately below.
                    let new_principal_trait_ref = rebound_super_trait_ref
                        .into_value_and_skipped_binders()
                        .0
                        .map(|it| it.cast(interner));

                    // Swap trait ref for `principal_a` with the new trait ref, drop the auto
                    // traits not included in the upcast target.
                    let new_source_ty = TyKind::Dyn(DynTy {
                        bounds: bounds_a.map_ref(|bounds| {
                            QuantifiedWhereClauses::from_iter(
                                interner,
                                bounds.iter(interner).cloned().filter_map(|bound| {
                                    let trait_id = match bound.trait_id() {
                                        Some(id) => id,
                                        None => return Some(bound),
                                    };

                                    if principal_a == trait_id {
                                        Some(new_principal_trait_ref.clone())
                                    } else {
                                        auto_trait_ids_b.contains(&trait_id).then_some(bound)
                                    }
                                }),
                            )
                        }),
                        lifetime: lifetime_b.clone(),
                    })
                    .intern(interner);

                    // Check that new source is equal to target
                    let eq_goal = EqGoal {
                        a: new_source_ty.cast(interner),
                        b: target_ty.clone().cast(interner),
                    }
                    .cast(interner);

                    // We don't push goal for `principal_b`'s object safety because it's implied by
                    // `principal_a`'s object safety.
                    builder
                        .push_clause(trait_ref.clone(), [eq_goal, lifetime_outlives_goal.clone()]);
                }
            }
        }

        // T -> dyn Trait + 'a
        (_, TyKind::Dyn(DynTy { bounds, lifetime })) => {
            // Check if all traits in trait object are object safe
            let object_safe_goals = bounds
                .skip_binders()
                .iter(interner)
                .filter_map(|bound| bound.trait_id())
                .map(|id| DomainGoal::ObjectSafe(id).cast(interner));

            // Check that T implements all traits of the trait object
            let source_ty_bounds = bounds
                .clone()
                .substitute(interner, &Substitution::from1(interner, source_ty.clone()));

            // Check that T is sized because we can only make
            // a trait object from a sized type
            let self_sized_goal: WhereClause<_> = TraitRef {
                trait_id: db
                    .well_known_trait_id(WellKnownTrait::Sized)
                    .expect("Expected Sized to be defined when proving Unsize"),
                substitution: Substitution::from1(interner, source_ty.clone()),
            }
            .cast(interner);

            // Check that `source_ty` outlives `'a`
            let source_ty_outlives: Goal<_> = WhereClause::TypeOutlives(TypeOutlives {
                ty: source_ty,
                lifetime: lifetime.clone(),
            })
            .cast(interner);

            builder.push_clause(
                trait_ref,
                source_ty_bounds
                    .iter(interner)
                    .map(|bound| bound.clone().cast::<Goal<I>>(interner))
                    .chain(object_safe_goals)
                    .chain(iter::once(self_sized_goal.cast(interner)))
                    .chain(iter::once(source_ty_outlives)),
            );
        }

        (TyKind::Array(array_ty, _array_const), TyKind::Slice(slice_ty)) => {
            let eq_goal = EqGoal {
                a: array_ty.clone().cast(interner),
                b: slice_ty.clone().cast(interner),
            };

            builder.push_clause(trait_ref, iter::once(eq_goal));
        }

        // Adt<T> -> Adt<U>
        (TyKind::Adt(adt_id_a, substitution_a), TyKind::Adt(adt_id_b, substitution_b)) => {
            if adt_id_a != adt_id_b {
                return;
            }

            let adt_id = *adt_id_a;
            let adt_datum = db.adt_datum(adt_id);

            // Unsizing of enums is not allowed
            if adt_datum.kind == AdtKind::Enum {
                return;
            }

            // We have a `struct` so we're guaranteed a single variant
            let fields_len = adt_datum
                .binders
                .skip_binders()
                .variants
                .last()
                .unwrap()
                .fields
                .len();

            if fields_len == 0 {
                return;
            }

            let adt_tail_field = adt_datum
                .binders
                .map_ref(|bound| bound.variants.last().unwrap().fields.last().unwrap())
                .cloned();

            // Collect unsize parameters that last field contains and
            // ensure there at least one of them.
            let unsize_parameter_candidates =
                outer_binder_parameters_used(interner, &adt_tail_field);

            if unsize_parameter_candidates.is_empty() {
                return;
            }
            // Ensure none of the other fields mention the parameters used
            // in unsizing.
            // We specifically want variables specified by the outermost binder
            // i.e. the struct generic arguments binder.
            if uses_outer_binder_params(
                interner,
                &adt_datum
                    .binders
                    .map_ref(|bound| &bound.variants.last().unwrap().fields[..fields_len - 1]),
                &unsize_parameter_candidates,
            ) {
                return;
            }

            let parameters_a = substitution_a.as_slice(interner);
            let parameters_b = substitution_b.as_slice(interner);
            // Check that the source adt with the target's
            // unsizing parameters is equal to the target.
            // We construct a new substitution where if a parameter is used in the
            // coercion (i.e. it's a non-lifetime struct parameter used by it's last field),
            // then we take that parameter from target substitution, otherwise we take
            // it from the source substitution.
            //
            // In order for the coercion to be valid, target struct and
            // struct with this newly constructed substitution applied to it should be equal.
            let substitution = Substitution::from_iter(
                interner,
                parameters_a.iter().enumerate().map(|(i, p)| {
                    if unsize_parameter_candidates.contains(&i) {
                        &parameters_b[i]
                    } else {
                        p
                    }
                }),
            );

            let eq_goal = EqGoal {
                a: TyKind::Adt(adt_id, substitution)
                    .intern(interner)
                    .cast(interner),
                b: target_ty.clone().cast(interner),
            }
            .cast(interner);

            // Extract `TailField<T>` and `TailField<U>` from `Struct<T>` and `Struct<U>`.
            let source_tail_field = adt_tail_field.clone().substitute(interner, substitution_a);
            let target_tail_field = adt_tail_field.substitute(interner, substitution_b);

            // Check that `TailField<T>: Unsize<TailField<U>>`
            let last_field_unsizing_goal: Goal<I> = TraitRef {
                trait_id: unsize_trait_id,
                substitution: Substitution::from_iter(
                    interner,
                    [source_tail_field, target_tail_field].iter().cloned(),
                ),
            }
            .cast(interner);

            builder.push_clause(trait_ref, [eq_goal, last_field_unsizing_goal].iter());
        }

        // (.., T) -> (.., U)
        (TyKind::Tuple(arity_a, substitution_a), TyKind::Tuple(arity_b, substitution_b)) => {
            if arity_a != arity_b || *arity_a == 0 {
                return;
            }
            let arity = arity_a;

            let tail_ty_a = substitution_a.iter(interner).last().unwrap();
            let tail_ty_b = substitution_b.iter(interner).last().unwrap();

            // Check that the source tuple with the target's
            // last element is equal to the target.
            let new_tuple = TyKind::Tuple(
                *arity,
                Substitution::from_iter(
                    interner,
                    substitution_a
                        .iter(interner)
                        .take(arity - 1)
                        .chain(iter::once(tail_ty_b)),
                ),
            )
            .cast(interner)
            .intern(interner);

            let eq_goal: Goal<I> = EqGoal {
                a: new_tuple.cast(interner),
                b: target_ty.clone().cast(interner),
            }
            .cast(interner);

            // Check that `T: Unsize<U>`
            let last_field_unsizing_goal: Goal<I> = TraitRef {
                trait_id: unsize_trait_id,
                substitution: Substitution::from_iter(
                    interner,
                    [tail_ty_a, tail_ty_b].iter().cloned(),
                ),
            }
            .cast(interner);

            builder.push_clause(trait_ref, [eq_goal, last_field_unsizing_goal].iter());
        }

        _ => (),
    }
}
