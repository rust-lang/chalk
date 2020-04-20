use std::collections::HashSet;
use std::iter;

use crate::clauses::ClauseBuilder;
use crate::{Interner, RustIrDatabase, TraitRef, WellKnownTrait};
use chalk_ir::{
    cast::Cast,
    interner::HasInterner,
    visit::{visitors::FindAny, SuperVisit, Visit, VisitResult, Visitor},
    ApplicationTy, Binders, DebruijnIndex, DomainGoal, DynTy, EqGoal, Goal, QuantifiedWhereClauses,
    Substitution, TraitId, Ty, TyData, TypeName, WhereClause,
};

struct UnsizeParameterCollector<'a, I: Interner> {
    interner: &'a I,
    // FIXME should probably use a bitset instead
    parameters: HashSet<usize>,
}

impl<'a, I: Interner> Visitor<'a, I> for UnsizeParameterCollector<'a, I> {
    type Result = ();

    fn as_dyn(&mut self) -> &mut dyn Visitor<'a, I, Result = Self::Result> {
        self
    }

    // FIXME(areredify) when const generics land, collect const variables too

    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> Self::Result {
        let interner = self.interner;

        match ty.data(interner) {
            TyData::BoundVar(bound_var) => {
                // check if bound var referse to the outermost binder
                if bound_var.debruijn.shifted_in() == outer_binder {
                    self.parameters.insert(bound_var.index);
                }
            }
            _ => ty.super_visit_with(self, outer_binder),
        }
    }

    fn interner(&self) -> &'a I {
        self.interner
    }
}

fn outer_binder_parameters_used<I: Interner>(
    interner: &I,
    v: &Binders<impl Visit<I> + HasInterner>,
) -> HashSet<usize> {
    let mut visitor = UnsizeParameterCollector {
        interner,
        parameters: HashSet::new(),
    };
    v.visit_with(&mut visitor, DebruijnIndex::INNERMOST);
    visitor.parameters
}

// has nothing to do with occurs check
struct ParameterOccurenceCheck<'a, 'p, I: Interner> {
    interner: &'a I,
    parameters: &'p HashSet<usize>,
}

impl<'a, 'p, I: Interner> Visitor<'a, I> for ParameterOccurenceCheck<'a, 'p, I> {
    type Result = FindAny;

    fn as_dyn(&mut self) -> &mut dyn Visitor<'a, I, Result = Self::Result> {
        self
    }

    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> Self::Result {
        let interner = self.interner;

        match ty.data(interner) {
            TyData::BoundVar(bound_var) => {
                if bound_var.debruijn.shifted_in() == outer_binder
                    && self.parameters.contains(&bound_var.index)
                {
                    FindAny::FOUND
                } else {
                    FindAny::new()
                }
            }
            _ => ty.super_visit_with(self, outer_binder),
        }
    }

    fn interner(&self) -> &'a I {
        self.interner
    }
}

fn uses_outer_binder_params<I: Interner>(
    interner: &I,
    v: &Binders<impl Visit<I> + HasInterner>,
    parameters: &HashSet<usize>,
) -> bool {
    let mut visitor = ParameterOccurenceCheck {
        interner,
        parameters,
    };
    v.visit_with(&mut visitor, DebruijnIndex::INNERMOST) == FindAny::FOUND
}

fn principal_id<'a, I: Interner>(
    db: &dyn RustIrDatabase<I>,
    bounds: &'a Binders<QuantifiedWhereClauses<I>>,
) -> Option<TraitId<I>> {
    let interner = db.interner();

    let principal_id = bounds
        .skip_binders()
        .iter(interner)
        .next()
        .expect("Expected trait object to have at least one trait bound")
        .trait_id()?;

    if db.trait_datum(principal_id).is_auto_trait() {
        None
    } else {
        Some(principal_id)
    }
}

fn auto_trait_ids<'a, I: Interner>(
    db: &dyn RustIrDatabase<I>,
    bounds: &'a Binders<QuantifiedWhereClauses<I>>,
) -> impl Iterator<Item = TraitId<I>> + 'a {
    let interner = db.interner();
    // all trait ref where clauses after the principal are auto traits
    let to_skip = if principal_id(db, bounds).is_some() {
        1
    } else {
        0
    };
    bounds
        .skip_binders()
        .iter(interner)
        .skip(to_skip)
        .filter_map(|clause| clause.trait_id())
}

pub fn add_unsize_program_clauses<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    builder: &mut ClauseBuilder<'_, I>,
    trait_ref: &TraitRef<I>,
    _ty: &TyData<I>,
) {
    let interner = db.interner();

    let source_ty = trait_ref.self_type_parameter(interner);
    let target_ty = trait_ref
        .substitution
        .at(interner, 1)
        .assert_ty_ref(interner);

    let unsize_trait_id = trait_ref.trait_id;

    // N.B. here rustc asserts that `TraitRef` is not a higher-ranked bound
    // i.e. `for<'a> &'a T: Unsize<dyn Trait+'a>` is never provable.
    //
    // In chalk it would be awkward to implement and I am not sure
    // there is a need for it, the original comment states that this restriction
    // could be lifted.
    //
    // for more info visit `fn assemble_candidates_for_unsizing` and
    // `fn confirm_builtin_unisize_candidate` in rustc.

    match (source_ty.data(interner), target_ty.data(interner)) {
        // dyn Trait + AutoX + 'a -> dyn Trait + AutoY + 'b
        (
            TyData::Dyn(DynTy {
                bounds: bounds_a,
                lifetime: lifetime_a,
            }),
            TyData::Dyn(DynTy {
                bounds: bounds_b,
                lifetime: lifetime_b,
            }),
        ) => {
            let principal_a = principal_id(db, bounds_a);
            let principal_b = principal_id(db, bounds_b);

            let auto_trait_ids_a: Vec<_> = auto_trait_ids(db, bounds_a).collect();
            let auto_trait_ids_b: Vec<_> = auto_trait_ids(db, bounds_b).collect();

            let may_apply = principal_a == principal_b
                && auto_trait_ids_b
                    .iter()
                    .all(|id_b| auto_trait_ids_a.iter().any(|id_a| id_a == id_b));

            if !may_apply {
                return;
            }

            // COMMENT FROM RUSTC:
            // Require that the traits involved in this upcast are **equal**;
            // only the **lifetime bound** is changed.
            //
            // FIXME: This condition is arguably too strong -- it would
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

            // Filter out auto traits of source that are not present in target
            // and change source lifetime to target lifetime
            //
            // This new type should be equal to target type.
            let source_ty = TyData::Dyn(DynTy {
                bounds: bounds_a.map_ref(|bounds| {
                    QuantifiedWhereClauses::from(
                        interner,
                        bounds.iter(interner).filter(|bound| {
                            let trait_id = match bound.trait_id() {
                                Some(id) => id,
                                None => return true,
                            };

                            if auto_trait_ids_a.iter().all(|&id_a| id_a != trait_id) {
                                return true;
                            }
                            auto_trait_ids_b.iter().any(|&id_b| id_b == trait_id)
                        }),
                    )
                }),
                lifetime: lifetime_b.clone(),
            })
            .intern(interner);

            // Check that new source is equal to target
            let eq_goal = EqGoal {
                a: source_ty.cast(interner),
                b: target_ty.clone().cast(interner),
            }
            .cast(interner);

            // FIXME(areredify) change this to outlives once #419 lands
            let lifetime_outlives_goal = EqGoal {
                a: lifetime_a.clone().cast(interner),
                b: lifetime_b.clone().cast(interner),
            }
            .cast(interner);

            builder.push_clause(trait_ref.clone(), [eq_goal, lifetime_outlives_goal].iter());
        }

        // T -> dyn Trait + 'a
        (
            _,
            TyData::Dyn(DynTy {
                bounds,
                lifetime: _,
            }),
        ) => {
            // Check if all traits in trait object are object safe
            let object_safe_goals = bounds
                .skip_binders()
                .iter(interner)
                .filter_map(|bound| bound.trait_id())
                .map(|id| DomainGoal::ObjectSafe(id).cast(interner));

            // Check that T implements all traits of the trait object
            let source_ty_bounds =
                bounds.substitute(interner, &Substitution::from1(interner, source_ty.clone()));

            // Check that T is sized because we can only make
            // a trait object from a sized type
            let self_sized_goal: WhereClause<_> = TraitRef {
                trait_id: db
                    .well_known_trait_id(WellKnownTrait::Sized)
                    .expect("Expected Sized to be defined when proving Unsize"),
                substitution: Substitution::from1(interner, source_ty.clone()),
            }
            .cast(interner);

            // FIXME(areredify) we need a `source_ty: 'lifetime` goal here

            builder.push_clause(
                trait_ref.clone(),
                source_ty_bounds
                    .iter(interner)
                    .map(|bound| bound.clone().cast::<Goal<I>>(interner))
                    .chain(object_safe_goals)
                    .chain(iter::once(self_sized_goal.cast(interner))),
            );
        }

        // Struct<T> -> Struct<U>
        // Unsizing of enums is not allowed
        (
            TyData::Apply(ApplicationTy {
                name: TypeName::Adt(struct_id_a),
                substitution: substitution_a,
            }),
            TyData::Apply(ApplicationTy {
                name: TypeName::Adt(struct_id_b),
                substitution: substitution_b,
            }),
        ) => {
            if struct_id_a != struct_id_b {
                return;
            }

            let struct_id = *struct_id_a;
            let struct_datum = db.adt_datum(struct_id);
            let fields_len = struct_datum.binders.skip_binders().fields.len();

            if fields_len == 0 {
                return;
            }

            let struct_tail_field = struct_datum
                .binders
                .map_ref(|bound| bound.fields.last().unwrap());

            // Collect unsize parameters that last field contains and
            // ensure there at least one of them.
            let unsize_parameter_candidates =
                outer_binder_parameters_used(interner, &struct_tail_field);

            if unsize_parameter_candidates.len() == 0 {
                return;
            }

            // Ensure none of the other fields mention the parameters used
            // in unsizing.
            if uses_outer_binder_params(
                interner,
                &struct_datum
                    .binders
                    .map_ref(|bound| &bound.fields[..fields_len - 1]),
                &unsize_parameter_candidates,
            ) {
                return;
            }

            let parameters_a = substitution_a.parameters(interner);
            let parameters_b = substitution_b.parameters(interner);
            // Check that the source struct with the target's
            // unsizing parameters is equal to the target.
            let substitution = Substitution::from(
                interner,
                parameters_a.iter().enumerate().map(|(i, p)| {
                    if unsize_parameter_candidates.contains(&i) {
                        p
                    } else {
                        &parameters_b[i]
                    }
                }),
            );

            let eq_goal = EqGoal {
                a: TyData::Apply(ApplicationTy {
                    name: TypeName::Adt(struct_id),
                    substitution,
                })
                .intern(interner)
                .cast(interner),
                b: target_ty.clone().cast(interner),
            }
            .cast(interner);

            // Extract `TailField<T>` and `TailField<U>` from `Struct<T>` and `Struct<U>`.
            let source_tail_field = struct_tail_field.substitute(interner, substitution_a);
            let target_tail_field = struct_tail_field.substitute(interner, substitution_b);

            // Check that `TailField<T>: Unsize<TailField<U>>`
            let last_field_unsizing_goal: Goal<I> = TraitRef {
                trait_id: unsize_trait_id,
                substitution: Substitution::from(
                    interner,
                    [source_tail_field, target_tail_field].iter().cloned(),
                ),
            }
            .cast(interner);

            builder.push_clause(
                trait_ref.clone(),
                [eq_goal, last_field_unsizing_goal].iter(),
            );
        }

        // (.., T) -> (.., U)
        (
            TyData::Apply(ApplicationTy {
                name: TypeName::Tuple(arity_a),
                substitution: substitution_a,
            }),
            TyData::Apply(ApplicationTy {
                name: TypeName::Tuple(arity_b),
                substitution: substitution_b,
            }),
        ) => {
            if arity_a != arity_b || *arity_a == 0 {
                return;
            }
            let arity = arity_a;

            let tail_ty_a = substitution_a.iter(interner).last().unwrap();
            let tail_ty_b = substitution_b.iter(interner).last().unwrap();

            // Check that the source tuple with the target's
            // last element is equal to the target.
            let new_tuple = ApplicationTy {
                name: TypeName::Tuple(*arity),
                substitution: Substitution::from(
                    interner,
                    substitution_a
                        .iter(interner)
                        .take(arity - 1)
                        .chain(iter::once(tail_ty_b)),
                ),
            }
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
                substitution: Substitution::from(interner, [tail_ty_a, tail_ty_b].iter().cloned()),
            }
            .cast(interner);

            builder.push_clause(
                trait_ref.clone(),
                [eq_goal, last_field_unsizing_goal].iter(),
            );
        }

        // FIXME(areredify) extend with array -> slice unsizing
        _ => (),
    }
}
