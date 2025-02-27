use std::ops::ControlFlow;
use std::{fmt, iter};

use crate::{
    ext::*, goal_builder::GoalBuilder, rust_ir::*, solve::Solver, split::Split, RustIrDatabase,
};
use chalk_ir::{
    cast::*,
    fold::shift::Shift,
    interner::Interner,
    visit::{TypeVisitable, TypeVisitor},
    *,
};
use tracing::debug;

#[derive(Debug)]
pub enum WfError<I: Interner> {
    IllFormedTypeDecl(chalk_ir::AdtId<I>),
    IllFormedOpaqueTypeDecl(chalk_ir::OpaqueTyId<I>),
    IllFormedTraitImpl(chalk_ir::TraitId<I>),
}

impl<I: Interner> fmt::Display for WfError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WfError::IllFormedTypeDecl(id) => write!(
                f,
                "type declaration `{:?}` does not meet well-formedness requirements",
                id
            ),
            WfError::IllFormedOpaqueTypeDecl(id) => write!(
                f,
                "opaque type declaration `{:?}` does not meet well-formedness requirements",
                id
            ),
            WfError::IllFormedTraitImpl(id) => write!(
                f,
                "trait impl for `{:?}` does not meet well-formedness requirements",
                id
            ),
        }
    }
}

impl<I: Interner> std::error::Error for WfError<I> {}

pub struct WfSolver<'a, I: Interner> {
    db: &'a dyn RustIrDatabase<I>,
    solver_builder: &'a dyn Fn() -> Box<dyn Solver<I>>,
}

struct InputTypeCollector<I: Interner> {
    types: Vec<Ty<I>>,
    interner: I,
}

impl<I: Interner> InputTypeCollector<I> {
    fn new(interner: I) -> Self {
        Self {
            types: Vec::new(),
            interner,
        }
    }

    fn types_in(interner: I, value: impl TypeVisitable<I>) -> Vec<Ty<I>> {
        let mut collector = Self::new(interner);
        value.visit_with(&mut collector, DebruijnIndex::INNERMOST);
        collector.types
    }
}

impl<I: Interner> TypeVisitor<I> for InputTypeCollector<I> {
    type BreakTy = ();
    fn as_dyn(&mut self) -> &mut dyn TypeVisitor<I, BreakTy = Self::BreakTy> {
        self
    }

    fn interner(&self) -> I {
        self.interner
    }

    fn visit_where_clause(
        &mut self,
        where_clause: &WhereClause<I>,
        outer_binder: DebruijnIndex,
    ) -> ControlFlow<()> {
        match where_clause {
            WhereClause::AliasEq(alias_eq) => alias_eq
                .alias
                .clone()
                .intern(self.interner)
                .visit_with(self, outer_binder),
            WhereClause::Implemented(trait_ref) => trait_ref.visit_with(self, outer_binder),
            WhereClause::TypeOutlives(TypeOutlives { ty, .. }) => ty.visit_with(self, outer_binder),
            WhereClause::LifetimeOutlives(..) => ControlFlow::Continue(()),
        }
    }

    fn visit_ty(&mut self, ty: &Ty<I>, outer_binder: DebruijnIndex) -> ControlFlow<()> {
        let interner = self.interner();

        let mut push_ty = || {
            self.types
                .push(ty.clone().shifted_out_to(interner, outer_binder).unwrap())
        };
        match ty.kind(interner) {
            TyKind::Adt(id, substitution) => {
                push_ty();
                id.visit_with(self, outer_binder);
                substitution.visit_with(self, outer_binder)
            }
            TyKind::AssociatedType(assoc_ty, substitution) => {
                push_ty();
                assoc_ty.visit_with(self, outer_binder);
                substitution.visit_with(self, outer_binder)
            }
            TyKind::Scalar(scalar) => {
                push_ty();
                scalar.visit_with(self, outer_binder)
            }
            TyKind::Str => {
                push_ty();
                ControlFlow::Continue(())
            }
            TyKind::Tuple(arity, substitution) => {
                push_ty();
                arity.visit_with(self, outer_binder);
                substitution.visit_with(self, outer_binder)
            }
            TyKind::OpaqueType(opaque_ty, substitution) => {
                push_ty();
                opaque_ty.visit_with(self, outer_binder);
                substitution.visit_with(self, outer_binder)
            }
            TyKind::Slice(substitution) => {
                push_ty();
                substitution.visit_with(self, outer_binder)
            }
            TyKind::FnDef(fn_def, substitution) => {
                push_ty();
                fn_def.visit_with(self, outer_binder);
                substitution.visit_with(self, outer_binder)
            }
            TyKind::Ref(mutability, lifetime, ty) => {
                push_ty();
                mutability.visit_with(self, outer_binder);
                lifetime.visit_with(self, outer_binder);
                ty.visit_with(self, outer_binder)
            }
            TyKind::Raw(mutability, substitution) => {
                push_ty();
                mutability.visit_with(self, outer_binder);
                substitution.visit_with(self, outer_binder)
            }
            TyKind::Never => {
                push_ty();
                ControlFlow::Continue(())
            }
            TyKind::Array(ty, const_) => {
                push_ty();
                ty.visit_with(self, outer_binder);
                const_.visit_with(self, outer_binder)
            }
            TyKind::Closure(_id, substitution) => {
                push_ty();
                substitution.visit_with(self, outer_binder)
            }
            TyKind::Coroutine(_coroutine, substitution) => {
                push_ty();
                substitution.visit_with(self, outer_binder)
            }
            TyKind::CoroutineWitness(_witness, substitution) => {
                push_ty();
                substitution.visit_with(self, outer_binder)
            }
            TyKind::Foreign(_foreign_ty) => {
                push_ty();
                ControlFlow::Continue(())
            }
            TyKind::Error => {
                push_ty();
                ControlFlow::Continue(())
            }

            TyKind::Dyn(clauses) => {
                push_ty();
                clauses.visit_with(self, outer_binder)
            }

            TyKind::Alias(AliasTy::Projection(proj)) => {
                push_ty();
                proj.visit_with(self, outer_binder)
            }

            TyKind::Alias(AliasTy::Opaque(opaque_ty)) => {
                push_ty();
                opaque_ty.visit_with(self, outer_binder)
            }

            TyKind::Placeholder(_) => {
                push_ty();
                ControlFlow::Continue(())
            }

            // Type parameters do not carry any input types (so we can sort of assume they are
            // always WF).
            TyKind::BoundVar(..) => ControlFlow::Continue(()),

            // Higher-kinded types such as `for<'a> fn(&'a u32)` introduce their own implied
            // bounds, and these bounds will be enforced upon calling such a function. In some
            // sense, well-formedness requirements for the input types of an HKT will be enforced
            // lazily, so no need to include them here.
            TyKind::Function(..) => ControlFlow::Continue(()),

            TyKind::InferenceVar(..) => {
                panic!("unexpected inference variable in wf rules: {:?}", ty)
            }
        }
    }
}

impl<'a, I> WfSolver<'a, I>
where
    I: Interner,
{
    /// Constructs a new `WfSolver`.
    pub fn new(
        db: &'a dyn RustIrDatabase<I>,
        solver_builder: &'a dyn Fn() -> Box<dyn Solver<I>>,
    ) -> Self {
        Self { db, solver_builder }
    }

    pub fn verify_adt_decl(&self, adt_id: AdtId<I>) -> Result<(), WfError<I>> {
        let interner = self.db.interner();

        // Given a struct like
        //
        // ```rust
        // struct Foo<T> where T: Eq {
        //     data: Vec<T>
        // }
        // ```
        let adt_datum = self.db.adt_datum(adt_id);
        let is_enum = adt_datum.kind == AdtKind::Enum;

        let mut gb = GoalBuilder::new(self.db);
        let adt_data = adt_datum
            .binders
            .map_ref(|b| (&b.variants, &b.where_clauses));

        // We make a goal like...
        //
        // forall<T> { ... }
        let wg_goal = gb.forall(
            &adt_data,
            is_enum,
            |gb, _, (variants, where_clauses), is_enum| {
                let interner = gb.interner();

                // (FromEnv(T: Eq) => ...)
                gb.implies(
                    where_clauses
                        .iter()
                        .cloned()
                        .map(|wc| wc.into_from_env_goal(interner)),
                    |gb| {
                        let sub_goals: Vec<_> = variants
                            .iter()
                            .flat_map(|variant| {
                                let fields = &variant.fields;

                                // When checking if Enum is well-formed, we require that all fields of
                                // each variant are sized. For `structs`, we relax this requirement to
                                // all but the last field.
                                let sized_constraint_goal =
                                    WfWellKnownConstraints::struct_sized_constraint(
                                        gb.db(),
                                        fields,
                                        is_enum,
                                    );

                                // WellFormed(Vec<T>), for each field type `Vec<T>` or type that appears in the where clauses
                                let types = InputTypeCollector::types_in(
                                    gb.interner(),
                                    (&fields, &where_clauses),
                                );

                                types
                                    .into_iter()
                                    .map(|ty| ty.well_formed().cast(interner))
                                    .chain(sized_constraint_goal.into_iter())
                            })
                            .collect();

                        gb.all(sub_goals)
                    },
                )
            },
        );

        let wg_goal = wg_goal.into_closed_goal(interner);
        let mut fresh_solver = (self.solver_builder)();
        let is_legal = fresh_solver.has_unique_solution(self.db, &wg_goal);

        if !is_legal {
            Err(WfError::IllFormedTypeDecl(adt_id))
        } else {
            Ok(())
        }
    }

    pub fn verify_trait_impl(&self, impl_id: ImplId<I>) -> Result<(), WfError<I>> {
        let interner = self.db.interner();

        let impl_datum = self.db.impl_datum(impl_id);
        let trait_id = impl_datum.trait_id();

        let impl_goal = Goal::all(
            interner,
            impl_header_wf_goal(self.db, impl_id).into_iter().chain(
                impl_datum
                    .associated_ty_value_ids
                    .iter()
                    .filter_map(|&id| compute_assoc_ty_goal(self.db, id)),
            ),
        );

        if let Some(well_known) = self.db.trait_datum(trait_id).well_known {
            self.verify_well_known_impl(impl_id, well_known)?
        }

        debug!("WF trait goal: {:?}", impl_goal);

        let mut fresh_solver = (self.solver_builder)();
        let is_legal =
            fresh_solver.has_unique_solution(self.db, &impl_goal.into_closed_goal(interner));

        if is_legal {
            Ok(())
        } else {
            Err(WfError::IllFormedTraitImpl(trait_id))
        }
    }

    pub fn verify_opaque_ty_decl(&self, opaque_ty_id: OpaqueTyId<I>) -> Result<(), WfError<I>> {
        // Given an opaque type like
        // ```notrust
        // opaque type Foo<T>: Clone where T: Bar = Baz;
        // ```
        let interner = self.db.interner();

        let mut gb = GoalBuilder::new(self.db);

        let datum = self.db.opaque_ty_data(opaque_ty_id);
        let bound = &datum.bound;

        // We make a goal like
        //
        // forall<T>
        let goal = gb.forall(bound, opaque_ty_id, |gb, _, bound, opaque_ty_id| {
            let interner = gb.interner();

            let subst = Substitution::from1(interner, gb.db().hidden_opaque_type(opaque_ty_id));

            let bounds = bound.bounds.clone().substitute(interner, &subst);
            let where_clauses = bound.where_clauses.clone().substitute(interner, &subst);

            let clauses = where_clauses
                .iter()
                .cloned()
                .map(|wc| wc.into_from_env_goal(interner));

            // if (WellFormed(T: Bar))
            gb.implies(clauses, |gb| {
                let interner = gb.interner();

                // all(WellFormed(Baz: Clone))
                gb.all(
                    bounds
                        .iter()
                        .cloned()
                        .map(|b| b.into_well_formed_goal(interner)),
                )
            })
        });

        debug!("WF opaque type goal: {:#?}", goal);

        let mut new_solver = (self.solver_builder)();
        let is_legal = new_solver.has_unique_solution(self.db, &goal.into_closed_goal(interner));

        if is_legal {
            Ok(())
        } else {
            Err(WfError::IllFormedOpaqueTypeDecl(opaque_ty_id))
        }
    }

    /// Verify builtin rules for well-known traits
    pub fn verify_well_known_impl(
        &self,
        impl_id: ImplId<I>,
        well_known: WellKnownTrait,
    ) -> Result<(), WfError<I>> {
        let mut solver = (self.solver_builder)();
        let impl_datum = self.db.impl_datum(impl_id);

        let is_legal = match well_known {
            WellKnownTrait::Copy => {
                WfWellKnownConstraints::copy_impl_constraint(&mut *solver, self.db, &impl_datum)
            }
            WellKnownTrait::Drop => {
                WfWellKnownConstraints::drop_impl_constraint(&mut *solver, self.db, &impl_datum)
            }
            WellKnownTrait::CoerceUnsized => {
                WfWellKnownConstraints::coerce_unsized_impl_constraint(
                    &mut *solver,
                    self.db,
                    &impl_datum,
                )
            }
            WellKnownTrait::DispatchFromDyn => {
                WfWellKnownConstraints::dispatch_from_dyn_constraint(
                    &mut *solver,
                    self.db,
                    &impl_datum,
                )
            }
            WellKnownTrait::Clone | WellKnownTrait::Unpin | WellKnownTrait::Future => true,
            // You can't add a manual implementation for the following traits:
            WellKnownTrait::Fn
            | WellKnownTrait::FnOnce
            | WellKnownTrait::FnMut
            | WellKnownTrait::AsyncFn
            | WellKnownTrait::AsyncFnOnce
            | WellKnownTrait::AsyncFnMut
            | WellKnownTrait::Unsize
            | WellKnownTrait::Sized
            | WellKnownTrait::DiscriminantKind
            | WellKnownTrait::Coroutine
            | WellKnownTrait::Pointee
            | WellKnownTrait::Tuple
            | WellKnownTrait::FnPtr => false,
        };

        if is_legal {
            Ok(())
        } else {
            Err(WfError::IllFormedTraitImpl(impl_datum.trait_id()))
        }
    }
}

fn impl_header_wf_goal<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    impl_id: ImplId<I>,
) -> Option<Goal<I>> {
    let impl_datum = db.impl_datum(impl_id);

    if !impl_datum.is_positive() {
        return None;
    }

    let impl_fields = impl_datum
        .binders
        .map_ref(|v| (&v.trait_ref, &v.where_clauses));

    let mut gb = GoalBuilder::new(db);
    // forall<P0...Pn> {...}
    let well_formed_goal = gb.forall(&impl_fields, (), |gb, _, (trait_ref, where_clauses), ()| {
        let interner = gb.interner();

        // if (WC && input types are well formed) { ... }
        gb.implies(
            impl_wf_environment(interner, where_clauses, trait_ref),
            |gb| {
                // We retrieve all the input types of the where clauses appearing on the trait impl,
                // e.g. in:
                // ```
                // impl<T, K> Foo for (T, K) where T: Iterator<Item = (HashSet<K>, Vec<Box<T>>)> { ... }
                // ```
                // we would retrieve `HashSet<K>`, `Box<T>`, `Vec<Box<T>>`, `(HashSet<K>, Vec<Box<T>>)`.
                // We will have to prove that these types are well-formed (e.g. an additional `K: Hash`
                // bound would be needed here).
                let types = InputTypeCollector::types_in(gb.interner(), &where_clauses);

                // Things to prove well-formed: input types of the where-clauses, projection types
                // appearing in the header, associated type values, and of course the trait ref.
                debug!(input_types=?types);
                let goals = types
                    .into_iter()
                    .map(|ty| ty.well_formed().cast(interner))
                    .chain(Some((*trait_ref).clone().well_formed().cast(interner)));

                gb.all::<_, Goal<I>>(goals)
            },
        )
    });

    Some(well_formed_goal)
}

/// Creates the conditions that an impl (and its contents of an impl)
/// can assume to be true when proving that it is well-formed.
fn impl_wf_environment<'i, I: Interner>(
    interner: I,
    where_clauses: &'i [QuantifiedWhereClause<I>],
    trait_ref: &'i TraitRef<I>,
) -> impl Iterator<Item = ProgramClause<I>> + 'i {
    // if (WC) { ... }
    let wc = where_clauses
        .iter()
        .cloned()
        .map(move |qwc| qwc.into_from_env_goal(interner).cast(interner));

    // We retrieve all the input types of the type on which we implement the trait: we will
    // *assume* that these types are well-formed, e.g. we will be able to derive that
    // `K: Hash` holds without writing any where clause.
    //
    // Example:
    // ```
    // struct HashSet<K> where K: Hash { ... }
    //
    // impl<K> Foo for HashSet<K> {
    //     // Inside here, we can rely on the fact that `K: Hash` holds
    // }
    // ```
    let types = InputTypeCollector::types_in(interner, trait_ref);

    let types_wf = types
        .into_iter()
        .map(move |ty| ty.into_from_env_goal(interner).cast(interner));

    wc.chain(types_wf)
}

/// Associated type values are special because they can be parametric (independently of
/// the impl), so we issue a special goal which is quantified using the binders of the
/// associated type value, for example in:
///
/// ```ignore
/// trait Foo {
///     type Item<'a>: Clone where Self: 'a
/// }
///
/// impl<T> Foo for Box<T> {
///     type Item<'a> = Box<&'a T>;
/// }
/// ```
///
/// we would issue the following subgoal: `forall<'a> { WellFormed(Box<&'a T>) }`.
///
/// Note that there is no binder for `T` in the above: the goal we
/// generate is expected to be exected in the context of the
/// larger WF goal for the impl, which already has such a
/// binder. So the entire goal for the impl might be:
///
/// ```ignore
/// forall<T> {
///     WellFormed(Box<T>) /* this comes from the impl, not this routine */,
///     forall<'a> { WellFormed(Box<&'a T>) },
/// }
/// ```
fn compute_assoc_ty_goal<I: Interner>(
    db: &dyn RustIrDatabase<I>,
    assoc_ty_id: AssociatedTyValueId<I>,
) -> Option<Goal<I>> {
    let mut gb = GoalBuilder::new(db);
    let assoc_ty = &db.associated_ty_value(assoc_ty_id);

    // Create `forall<T, 'a> { .. }`
    Some(gb.forall(
        &assoc_ty.value.map_ref(|v| &v.ty),
        assoc_ty_id,
        |gb, assoc_ty_substitution, value_ty, assoc_ty_id| {
            let interner = gb.interner();
            let db = gb.db();

            // Hmm, because `Arc<AssociatedTyValue>` does not implement `TypeFoldable`, we can't pass this value through,
            // just the id, so we have to fetch `assoc_ty` from the database again.
            // Implementing `TypeFoldable` for `AssociatedTyValue` doesn't *quite* seem right though, as that
            // would result in a deep clone, and the value is inert. We could do some more refatoring
            // (move the `Arc` behind a newtype, for example) to fix this, but for now doesn't
            // seem worth it.
            let assoc_ty = &db.associated_ty_value(assoc_ty_id);

            let (impl_parameters, projection) = db
                .impl_parameters_and_projection_from_associated_ty_value(
                    assoc_ty_substitution.as_slice(interner),
                    assoc_ty,
                );

            // If (/* impl WF environment */) { ... }
            let impl_id = assoc_ty.impl_id;
            let impl_datum = &db.impl_datum(impl_id);
            let ImplDatumBound {
                trait_ref: impl_trait_ref,
                where_clauses: impl_where_clauses,
            } = impl_datum
                .binders
                .clone()
                .substitute(interner, impl_parameters);
            let impl_wf_clauses =
                impl_wf_environment(interner, &impl_where_clauses, &impl_trait_ref);
            gb.implies(impl_wf_clauses, |gb| {
                // Get the bounds and where clauses from the trait
                // declaration, substituted appropriately.
                //
                // From our example:
                //
                // * bounds
                //     * original in trait, `Clone`
                //     * after substituting impl parameters, `Clone`
                //     * note that the self-type is not yet supplied for bounds,
                //       we will do that later
                // * where clauses
                //     * original in trait, `Self: 'a`
                //     * after substituting impl parameters, `Box<!T>: '!a`
                let assoc_ty_datum = db.associated_ty_data(projection.associated_ty_id);
                let AssociatedTyDatumBound {
                    bounds: defn_bounds,
                    where_clauses: defn_where_clauses,
                } = assoc_ty_datum
                    .binders
                    .clone()
                    .substitute(interner, &projection.substitution);

                // Create `if (/* where clauses on associated type value */) { .. }`
                gb.implies(
                    defn_where_clauses
                        .iter()
                        .cloned()
                        .map(|qwc| qwc.into_from_env_goal(interner)),
                    |gb| {
                        let types = InputTypeCollector::types_in(gb.interner(), value_ty);

                        // We require that `WellFormed(T)` for each type that appears in the value
                        let wf_goals = types
                            .into_iter()
                            .map(|ty| ty.well_formed())
                            .casted(interner);

                        // Check that the `value_ty` meets the bounds from the trait.
                        // Here we take the substituted bounds (`defn_bounds`) and we
                        // supply the self-type `value_ty` to yield the final result.
                        //
                        // In our example, the bound was `Clone`, so the combined
                        // result is `Box<!T>: Clone`. This is then converted to a
                        // well-formed goal like `WellFormed(Box<!T>: Clone)`.
                        let bound_goals = defn_bounds
                            .iter()
                            .cloned()
                            .flat_map(|qb| qb.into_where_clauses(interner, (*value_ty).clone()))
                            .map(|qwc| qwc.into_well_formed_goal(interner))
                            .casted(interner);

                        // Concatenate the WF goals of inner types + the requirements from trait
                        gb.all::<_, Goal<I>>(wf_goals.chain(bound_goals))
                    },
                )
            })
        },
    ))
}

/// Defines methods to compute well-formedness goals for well-known
/// traits (e.g. a goal for all fields of struct in a Copy impl to be Copy)
struct WfWellKnownConstraints;

impl WfWellKnownConstraints {
    /// Computes a goal to prove Sized constraints on a struct definition.
    /// Struct is considered well-formed (in terms of Sized) when it either
    /// has no fields or all of it's fields except the last are proven to be Sized.
    pub fn struct_sized_constraint<I: Interner>(
        db: &dyn RustIrDatabase<I>,
        fields: &[Ty<I>],
        size_all: bool,
    ) -> Option<Goal<I>> {
        let excluded = if size_all { 0 } else { 1 };

        if fields.len() <= excluded {
            return None;
        }

        let interner = db.interner();

        let sized_trait = db.well_known_trait_id(WellKnownTrait::Sized)?;

        Some(Goal::all(
            interner,
            fields[..fields.len() - excluded].iter().map(|ty| {
                TraitRef {
                    trait_id: sized_trait,
                    substitution: Substitution::from1(interner, ty.clone()),
                }
                .cast(interner)
            }),
        ))
    }

    /// Verify constraints on a Copy implementation.
    /// Copy impl is considered well-formed for
    ///    a) certain builtin types (scalar values, shared ref, etc..)
    ///    b) adts which
    ///        1) have all Copy fields
    ///        2) don't have a Drop impl
    fn copy_impl_constraint<I: Interner>(
        solver: &mut dyn Solver<I>,
        db: &dyn RustIrDatabase<I>,
        impl_datum: &ImplDatum<I>,
    ) -> bool {
        let interner = db.interner();

        let mut gb = GoalBuilder::new(db);

        let impl_fields = impl_datum
            .binders
            .map_ref(|v| (&v.trait_ref, &v.where_clauses));

        // Implementations for scalars, pointer types and never type are provided by libcore.
        // User implementations on types other than ADTs are forbidden.
        match impl_datum
            .binders
            .skip_binders()
            .trait_ref
            .self_type_parameter(interner)
            .kind(interner)
        {
            TyKind::Scalar(_)
            | TyKind::Raw(_, _)
            | TyKind::Ref(Mutability::Not, _, _)
            | TyKind::Never => return true,

            TyKind::Adt(_, _) => (),

            _ => return false,
        };

        // Well fomedness goal for ADTs
        let well_formed_goal =
            gb.forall(&impl_fields, (), |gb, _, (trait_ref, where_clauses), ()| {
                let interner = gb.interner();

                let ty = trait_ref.self_type_parameter(interner);

                let (adt_id, substitution) = match ty.kind(interner) {
                    TyKind::Adt(adt_id, substitution) => (*adt_id, substitution),

                    _ => unreachable!(),
                };

                // if (WC) { ... }
                gb.implies(
                    impl_wf_environment(interner, where_clauses, trait_ref),
                    |gb| -> Goal<I> {
                        let db = gb.db();

                        // not { Implemented(ImplSelfTy: Drop) }
                        let neg_drop_goal =
                            db.well_known_trait_id(WellKnownTrait::Drop)
                                .map(|drop_trait_id| {
                                    TraitRef {
                                        trait_id: drop_trait_id,
                                        substitution: Substitution::from1(interner, ty.clone()),
                                    }
                                    .cast::<Goal<I>>(interner)
                                    .negate(interner)
                                });

                        let adt_datum = db.adt_datum(adt_id);

                        let goals = adt_datum
                            .binders
                            .map_ref(|b| &b.variants)
                            .cloned()
                            .substitute(interner, substitution)
                            .into_iter()
                            .flat_map(|v| {
                                v.fields.into_iter().map(|f| {
                                    // Implemented(FieldTy: Copy)
                                    TraitRef {
                                        trait_id: trait_ref.trait_id,
                                        substitution: Substitution::from1(interner, f),
                                    }
                                    .cast(interner)
                                })
                            })
                            .chain(neg_drop_goal.into_iter());
                        gb.all(goals)
                    },
                )
            });

        solver.has_unique_solution(db, &well_formed_goal.into_closed_goal(interner))
    }

    /// Verifies constraints on a Drop implementation
    /// Drop implementation is considered well-formed if:
    ///     a) it's implemented on an ADT
    ///     b) The generic parameters of the impl's type must all be parameters
    ///        of the Drop impl itself (i.e., no specialization like
    ///        `impl Drop for S<Foo> {...}` is allowed).
    ///     c) Any bounds on the genereic parameters of the impl must be
    ///        deductible from the bounds imposed by the struct definition
    ///        (i.e. the implementation must be exactly as generic as the ADT definition).
    ///
    /// ```rust,ignore
    /// struct S<T1, T2> { }
    /// struct Foo<T> { }
    ///
    /// impl<U1: Copy, U2: Sized> Drop for S<U2, Foo<U1>> { }
    /// ```
    ///
    /// generates the following:
    /// goal derived from c):
    ///
    /// ```notrust
    /// forall<U1, U2> {
    ///    Implemented(U1: Copy), Implemented(U2: Sized) :- FromEnv(S<U2, Foo<U1>>)
    /// }
    /// ```
    ///
    /// goal derived from b):
    /// ```notrust
    /// forall <T1, T2> {
    ///     exists<U1, U2> {
    ///        S<T1, T2> = S<U2, Foo<U1>>
    ///     }
    /// }
    /// ```
    fn drop_impl_constraint<I: Interner>(
        solver: &mut dyn Solver<I>,
        db: &dyn RustIrDatabase<I>,
        impl_datum: &ImplDatum<I>,
    ) -> bool {
        let interner = db.interner();

        let adt_id = match impl_datum.self_type_adt_id(interner) {
            Some(id) => id,
            // Drop can only be implemented on a nominal type
            None => return false,
        };

        let mut gb = GoalBuilder::new(db);

        let adt_datum = db.adt_datum(adt_id);

        let impl_fields = impl_datum
            .binders
            .map_ref(|v| (&v.trait_ref, &v.where_clauses));

        // forall<ImplP1...ImplPn> { .. }
        let implied_by_adt_def_goal =
            gb.forall(&impl_fields, (), |gb, _, (trait_ref, where_clauses), ()| {
                let interner = gb.interner();

                // FromEnv(ImplSelfType) => ...
                gb.implies(
                    iter::once(
                        FromEnv::Ty(trait_ref.self_type_parameter(interner))
                            .cast::<DomainGoal<I>>(interner),
                    ),
                    |gb| {
                        // All(ImplWhereClauses)
                        gb.all(
                            where_clauses
                                .iter()
                                .map(|wc| wc.clone().into_well_formed_goal(interner)),
                        )
                    },
                )
            });

        let impl_self_ty = impl_datum
            .binders
            .map_ref(|b| b.trait_ref.self_type_parameter(interner));

        // forall<StructP1..StructPN> {...}
        let eq_goal = gb.forall(
            &adt_datum.binders,
            (adt_id, impl_self_ty),
            |gb, substitution, _, (adt_id, impl_self_ty)| {
                let interner = gb.interner();

                let def_adt = TyKind::Adt(adt_id, substitution).intern(interner);

                // exists<ImplP1...ImplPn> { .. }
                gb.exists(&impl_self_ty, def_adt, |gb, _, impl_adt, def_adt| {
                    let interner = gb.interner();

                    // StructName<StructP1..StructPn> = ImplSelfType
                    GoalData::EqGoal(EqGoal {
                        a: GenericArgData::Ty(def_adt).intern(interner),
                        b: GenericArgData::Ty(impl_adt.clone()).intern(interner),
                    })
                    .intern(interner)
                })
            },
        );

        let well_formed_goal = gb.all([implied_by_adt_def_goal, eq_goal].iter());

        solver.has_unique_solution(db, &well_formed_goal.into_closed_goal(interner))
    }

    /// Verify constraints a CoerceUnsized impl.
    /// Rules for CoerceUnsized impl to be considered well-formed:
    /// 1) pointer conversions: `&[mut] T -> &[mut] U`, `&[mut] T -> *[mut] U`,
    ///    `*[mut] T -> *[mut] U` are considered valid if
    ///    1) `T: Unsize<U>`
    ///    2) mutability is respected, i.e. immutable -> immutable, mutable -> immutable,
    ///       mutable -> mutable conversions are allowed, immutable -> mutable is not.
    /// 2) struct conversions of structures with the same definition, `S<P0...Pn>` -> `S<Q0...Qn>`.
    ///    To check if this impl is legal, we would walk down the fields of `S`
    ///    and consider their types with both substitutes. We are looking to find
    ///    exactly one (non-phantom) field that has changed its type (from `T` to `U`), and
    ///    expect `T` to be unsizeable to `U`, i.e. `T: CoerceUnsized<U>`.
    ///
    ///    As an example, consider a struct
    ///    ```rust
    ///    struct Foo<T, U> {
    ///        extra: T,
    ///        ptr: *mut U,
    ///    }
    ///    ```
    ///
    ///    We might have an impl that allows (e.g.) `Foo<T, [i32; 3]>` to be unsized
    ///    to `Foo<T, [i32]>`. That impl would look like:
    ///    ```rust,ignore
    ///    impl<T, U: Unsize<V>, V> CoerceUnsized<Foo<T, V>> for Foo<T, U> {}
    ///    ```
    ///    In this case:
    ///
    ///    - `extra` has type `T` before and type `T` after
    ///    - `ptr` has type `*mut U` before and type `*mut V` after
    ///
    ///    Since just one field changed, we would then check that `*mut U: CoerceUnsized<*mut V>`
    ///    is implemented. This will work out because `U: Unsize<V>`, and we have a libcore rule
    ///    that `*mut U` can be coerced to `*mut V` if `U: Unsize<V>`.
    fn coerce_unsized_impl_constraint<I: Interner>(
        solver: &mut dyn Solver<I>,
        db: &dyn RustIrDatabase<I>,
        impl_datum: &ImplDatum<I>,
    ) -> bool {
        let interner = db.interner();
        let mut gb = GoalBuilder::new(db);

        let (binders, impl_datum) = impl_datum.binders.as_ref().into();

        let trait_ref: &TraitRef<I> = &impl_datum.trait_ref;

        let source = trait_ref.self_type_parameter(interner);
        let target = trait_ref
            .substitution
            .at(interner, 1)
            .assert_ty_ref(interner)
            .clone();

        let mut place_in_environment = |goal| -> Goal<I> {
            gb.forall(
                &Binders::new(
                    binders.clone(),
                    (goal, trait_ref, &impl_datum.where_clauses),
                ),
                (),
                |gb, _, (goal, trait_ref, where_clauses), ()| {
                    let interner = gb.interner();
                    gb.implies(
                        impl_wf_environment(interner, where_clauses, trait_ref),
                        |_| goal,
                    )
                },
            )
        };

        match (source.kind(interner), target.kind(interner)) {
            (TyKind::Ref(s_m, _, source), TyKind::Ref(t_m, _, target))
            | (TyKind::Ref(s_m, _, source), TyKind::Raw(t_m, target))
            | (TyKind::Raw(s_m, source), TyKind::Raw(t_m, target)) => {
                if (*s_m, *t_m) == (Mutability::Not, Mutability::Mut) {
                    return false;
                }

                let unsize_trait_id =
                    if let Some(id) = db.well_known_trait_id(WellKnownTrait::Unsize) {
                        id
                    } else {
                        return false;
                    };

                // Source: Unsize<Target>
                let unsize_goal: Goal<I> = TraitRef {
                    trait_id: unsize_trait_id,
                    substitution: Substitution::from_iter(
                        interner,
                        [source.clone(), target.clone()].iter().cloned(),
                    ),
                }
                .cast(interner);

                // ImplEnv -> Source: Unsize<Target>
                let unsize_goal = place_in_environment(unsize_goal);

                solver.has_unique_solution(db, &unsize_goal.into_closed_goal(interner))
            }
            (TyKind::Adt(source_id, subst_a), TyKind::Adt(target_id, subst_b)) => {
                let adt_datum = db.adt_datum(*source_id);

                if source_id != target_id || adt_datum.kind != AdtKind::Struct {
                    return false;
                }

                let fields = adt_datum
                    .binders
                    .map_ref(|bound| &bound.variants.last().unwrap().fields)
                    .cloned();

                let (source_fields, target_fields) = (
                    fields.clone().substitute(interner, subst_a),
                    fields.substitute(interner, subst_b),
                );

                // collect fields with unequal ids
                let uneq_field_ids: Vec<usize> = (0..source_fields.len())
                    .filter(|&i| {
                        // ignore phantom data fields
                        if let Some(adt_id) = source_fields[i].adt_id(interner) {
                            if db.adt_datum(adt_id).flags.phantom_data {
                                return false;
                            }
                        }

                        let eq_goal: Goal<I> = EqGoal {
                            a: source_fields[i].clone().cast(interner),
                            b: target_fields[i].clone().cast(interner),
                        }
                        .cast(interner);

                        // ImplEnv -> Source.fields[i] = Target.fields[i]
                        let eq_goal = place_in_environment(eq_goal);

                        // We are interested in !UNEQUAL! fields
                        !solver.has_unique_solution(db, &eq_goal.into_closed_goal(interner))
                    })
                    .collect();

                if uneq_field_ids.len() != 1 {
                    return false;
                }

                let field_id = uneq_field_ids[0];

                // Source.fields[i]: CoerceUnsized<TargetFields[i]>
                let coerce_unsized_goal: Goal<I> = TraitRef {
                    trait_id: trait_ref.trait_id,
                    substitution: Substitution::from_iter(
                        interner,
                        [
                            source_fields[field_id].clone(),
                            target_fields[field_id].clone(),
                        ]
                        .iter()
                        .cloned(),
                    ),
                }
                .cast(interner);

                // ImplEnv -> Source.fields[i]: CoerceUnsized<TargetFields[i]>
                let coerce_unsized_goal = place_in_environment(coerce_unsized_goal);

                solver.has_unique_solution(db, &coerce_unsized_goal.into_closed_goal(interner))
            }
            _ => false,
        }
    }

    /// Verify constraints of a DispatchFromDyn impl.
    ///
    /// Rules for DispatchFromDyn impl to be considered well-formed:
    ///
    /// * Self and the type parameter must both be references or raw pointers with the same mutabilty
    /// * OR all the following hold:
    ///   - Self and the type parameter must be structs
    ///   - Self and the type parameter must have the same definitions
    ///   - Self must not be `#[repr(packed)]` or `#[repr(C)]`
    ///   - Self must have exactly one field which is not a 1-ZST (there may be any number of 1-ZST
    ///     fields), and that field must have a different type in the type parameter (i.e., it is
    ///     the field being coerced)
    ///   - `DispatchFromDyn` is implemented for the type of the field being coerced.
    fn dispatch_from_dyn_constraint<I: Interner>(
        solver: &mut dyn Solver<I>,
        db: &dyn RustIrDatabase<I>,
        impl_datum: &ImplDatum<I>,
    ) -> bool {
        let interner = db.interner();
        let mut gb = GoalBuilder::new(db);

        let (binders, impl_datum) = impl_datum.binders.as_ref().into();

        let trait_ref: &TraitRef<I> = &impl_datum.trait_ref;

        // DispatchFromDyn specifies that Self (source) can be coerced to T (target; its single type parameter).
        let source = trait_ref.self_type_parameter(interner);
        let target = trait_ref
            .substitution
            .at(interner, 1)
            .assert_ty_ref(interner)
            .clone();

        let mut place_in_environment = |goal| -> Goal<I> {
            gb.forall(
                &Binders::new(
                    binders.clone(),
                    (goal, trait_ref, &impl_datum.where_clauses),
                ),
                (),
                |gb, _, (goal, trait_ref, where_clauses), ()| {
                    let interner = gb.interner();
                    gb.implies(
                        impl_wf_environment(interner, &where_clauses, &trait_ref),
                        |_| goal,
                    )
                },
            )
        };

        match (source.kind(interner), target.kind(interner)) {
            (TyKind::Ref(s_m, _, _), TyKind::Ref(t_m, _, _))
            | (TyKind::Raw(s_m, _), TyKind::Raw(t_m, _))
                if s_m == t_m =>
            {
                true
            }
            (TyKind::Adt(source_id, subst_a), TyKind::Adt(target_id, subst_b)) => {
                let adt_datum = db.adt_datum(*source_id);

                // Definitions are equal and are structs.
                if source_id != target_id || adt_datum.kind != AdtKind::Struct {
                    return false;
                }

                // Not repr(C) or repr(packed).
                let repr = db.adt_repr(*source_id);
                if repr.c || repr.packed {
                    return false;
                }

                // Collect non 1-ZST fields; there must be exactly one.
                let fields = adt_datum
                    .binders
                    .map_ref(|bound| &bound.variants.last().unwrap().fields)
                    .cloned();

                let (source_fields, target_fields) = (
                    fields.clone().substitute(interner, subst_a),
                    fields.substitute(interner, subst_b),
                );

                let mut non_zst_fields: Vec<_> = source_fields
                    .iter()
                    .zip(target_fields.iter())
                    .filter(|(sf, _)| match sf.adt_id(interner) {
                        Some(adt) => !db.adt_size_align(adt).one_zst(),
                        None => true,
                    })
                    .collect();

                if non_zst_fields.len() != 1 {
                    return false;
                }

                // The field being coerced (the interesting field).
                let (field_src, field_tgt) = non_zst_fields.pop().unwrap();

                // The interesting field is different in the source and target types.
                let eq_goal: Goal<I> = EqGoal {
                    a: field_src.clone().cast(interner),
                    b: field_tgt.clone().cast(interner),
                }
                .cast(interner);
                let eq_goal = place_in_environment(eq_goal);
                if solver.has_unique_solution(db, &eq_goal.into_closed_goal(interner)) {
                    return false;
                }

                // Type(field_src): DispatchFromDyn<Type(field_tgt)>
                let field_dispatch_goal: Goal<I> = TraitRef {
                    trait_id: trait_ref.trait_id,
                    substitution: Substitution::from_iter(
                        interner,
                        [field_src.clone(), field_tgt.clone()].iter().cloned(),
                    ),
                }
                .cast(interner);
                let field_dispatch_goal = place_in_environment(field_dispatch_goal);
                if !solver.has_unique_solution(db, &field_dispatch_goal.into_closed_goal(interner))
                {
                    return false;
                }

                true
            }
            _ => false,
        }
    }
}
