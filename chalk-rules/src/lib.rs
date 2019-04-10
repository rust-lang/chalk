#[macro_use]
extern crate chalk_macros;
#[macro_use]
extern crate failure;

use chalk_ir::cast::{Cast, Caster};
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::Subst;
use chalk_ir::*;
use chalk_rust_ir::*;
use std::iter;
use std::sync::Arc;

pub mod coherence;
pub mod wf;

pub trait RustIrSource {
    /// Returns the datum for the associated type with the given id.
    fn associated_ty_data(&self, ty: TypeId) -> Arc<AssociatedTyDatum>;

    /// Returns the datum for the impl with the given id.
    fn trait_datum(&self, trait_id: TraitId) -> Arc<TraitDatum>;

    /// Returns the datum for the impl with the given id.
    fn struct_datum(&self, struct_id: StructId) -> Arc<StructDatum>;

    /// Returns the datum for the impl with the given id.
    fn impl_datum(&self, impl_id: ImplId) -> Arc<ImplDatum>;

    /// Returns all the impls for a given trait.
    fn impls_for_trait(&self, trait_id: TraitId) -> Vec<ImplId>;

    /// Returns true if there is an explicit impl of the auto trait
    /// `auto_trait_id` for the struct `struct_id`. This is part of
    /// the auto trait handling -- if there is no explicit impl given
    /// by the user for the struct, then we provide default impls
    /// based on the field types (otherwise, we rely on the impls the
    /// user gave).
    fn impl_provided_for(&self, auto_trait_id: TraitId, struct_id: StructId) -> bool;

    /// Returns the name for the type with the given id.
    fn type_name(&self, id: TypeKindId) -> Identifier;

    /// Given a projection of an associated type, splits the type
    /// parameters into two parts: those that come from the trait, and
    /// those that come from the associated type itself.
    ///
    /// e.g. given a projection `<Foo as Iterable>::Item<'x>`, where `Iterable` is defined like so:
    ///
    /// ```ignore
    /// trait Iterable { type Item<'a>; }
    /// ```
    ///
    /// we would split into the type parameter lists `[Foo]` (from the
    /// trait) and `['x]` (from the type).
    fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy,
    ) -> (Arc<AssociatedTyDatum>, &'p [Parameter], &'p [Parameter]);
}

pub trait ToProgramClauses {
    fn to_program_clauses(&self, program: &dyn RustIrSource, clauses: &mut Vec<ProgramClause>);
}

impl ToProgramClauses for ImplDatum {
    /// Given `impl<T: Clone> Clone for Vec<T> { ... }`, generate:
    ///
    /// ```notrust
    /// -- Rule Implemented-From-Impl
    /// forall<T> {
    ///     Implemented(Vec<T>: Clone) :- Implemented(T: Clone).
    /// }
    /// ```
    fn to_program_clauses(&self, _program: &dyn RustIrSource, clauses: &mut Vec<ProgramClause>) {
        clauses.push(
            self.binders
                .map_ref(|bound| ProgramClauseImplication {
                    consequence: bound.trait_ref.trait_ref().clone().cast(),
                    conditions: bound.where_clauses.iter().cloned().casted().collect(),
                })
                .cast(),
        );
    }
}

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
    auto_trait: &TraitDatum,
    struct_id: StructId,
    struct_datum: &StructDatum,
    program: &dyn RustIrSource,
    vec: &mut Vec<ProgramClause>,
) {
    // Must be an auto trait.
    assert!(auto_trait.binders.value.flags.auto);

    // Auto traits never have generic parameters of their own (apart from `Self`).
    assert_eq!(auto_trait.binders.binders.len(), 1);

    // If there is a `impl AutoTrait for Foo<..>` or `impl !AutoTrait
    // for Foo<..>`, where `Foo` is the struct we're looking at, then
    // we don't generate our own rules.
    if program.impl_provided_for(auto_trait_id, struct_id) {
        return;
    }

    vec.push({
        // trait_ref = `MyStruct<...>: MyAutoTrait`
        let auto_trait_ref = TraitRef {
            trait_id: auto_trait.binders.value.trait_ref.trait_id,
            parameters: vec![Ty::Apply(struct_datum.binders.value.self_ty.clone()).cast()],
        };

        // forall<P0..Pn> { // generic parameters from struct
        //   MyStruct<...>: MyAutoTrait :-
        //      Field0: MyAutoTrait,
        //      ...
        //      FieldN: MyAutoTrait
        // }
        struct_datum
            .binders
            .map_ref(|struct_contents| ProgramClauseImplication {
                consequence: auto_trait_ref.clone().cast(),
                conditions: struct_contents
                    .fields
                    .iter()
                    .cloned()
                    .map(|field_ty| TraitRef {
                        trait_id: auto_trait_id,
                        parameters: vec![field_ty.cast()],
                    })
                    .casted()
                    .collect(),
            })
            .cast()
    });
}

impl ToProgramClauses for AssociatedTyValue {
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
    /// -- Rule Normalize-From-Impl
    /// forall<'a, T> {
    ///     Normalize(<Vec<T> as Iterable>::IntoIter<'a> -> Iter<'a, T>>) :-
    ///         Implemented(Vec<T>: Iterable),  // (1)
    ///         Implemented(Iter<'a, T>: 'a).   // (2)
    /// }
    /// ```
    ///
    /// and:
    ///
    /// ```notrust
    /// forall<'a, T> {
    ///     UnselectedNormalize(Vec<T>::IntoIter<'a> -> Iter<'a, T>) :-
    ///         InScope(Iterable),
    ///         Normalize(<Vec<T> as Iterable>::IntoIter<'a> -> Iter<'a, T>).
    /// }
    /// ```
    fn to_program_clauses(&self, program: &dyn RustIrSource, clauses: &mut Vec<ProgramClause>) {
        let impl_datum = program.impl_datum(self.impl_id);
        let associated_ty = program.associated_ty_data(self.associated_ty_id);

        // Begin with the innermost parameters (`'a`) and then add those from impl (`T`).
        let all_binders: Vec<_> = self
            .value
            .binders
            .iter()
            .chain(impl_datum.binders.binders.iter())
            .cloned()
            .collect();

        let impl_trait_ref = impl_datum
            .binders
            .value
            .trait_ref
            .trait_ref()
            .shifted_in(self.value.len());

        let all_parameters: Vec<_> = self
            .value
            .binders
            .iter()
            .zip(0..)
            .map(|p| p.to_parameter())
            .chain(impl_trait_ref.parameters.iter().cloned())
            .collect();

        // Assemble the full list of conditions for projection to be valid.
        // This comes in two parts, marked as (1) and (2) in doc above:
        //
        // 1. require that the trait is implemented
        // 2. any where-clauses from the `type` declaration in the trait: the
        //    parameters must be substituted with those of the impl
        let where_clauses = associated_ty
            .where_clauses
            .iter()
            .map(|wc| Subst::apply(&all_parameters, wc))
            .casted();

        let conditions: Vec<Goal> = where_clauses
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
            parameters: parameters
                .iter()
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
        }
        .cast();

        let unselected_projection = UnselectedProjectionTy {
            type_name: associated_ty.name.clone(),
            parameters: parameters,
        };

        let unselected_normalization = Binders {
            binders: all_binders,
            value: ProgramClauseImplication {
                consequence: DomainGoal::UnselectedNormalize(UnselectedNormalize {
                    projection: unselected_projection,
                    ty: self.value.value.ty.clone(),
                }),
                conditions: vec![
                    normalize_goal.cast(),
                    DomainGoal::InScope(impl_trait_ref.trait_id.into()).cast(),
                ],
            },
        }
        .cast();

        clauses.push(normalization);
        clauses.push(unselected_normalization);
    }
}

impl ToProgramClauses for StructDatum {
    /// Given the following type definition: `struct Foo<T: Eq> { }`, generate:
    ///
    /// ```notrust
    /// -- Rule WellFormed-Type
    /// forall<T> {
    ///     WF(Foo<T>) :- WF(T: Eq).
    /// }
    ///
    /// -- Rule Implied-Bound-From-Type
    /// forall<T> {
    ///     FromEnv(T: Eq) :- FromEnv(Foo<T>).
    /// }
    ///
    /// forall<T> {
    ///     IsFullyVisible(Foo<T>) :- IsFullyVisible(T).
    /// }
    /// ```
    ///
    /// If the type `Foo` is marked `#[upstream]`, we also generate:
    ///
    /// ```notrust
    /// forall<T> { IsUpstream(Foo<T>). }
    /// ```
    ///
    /// Otherwise, if the type `Foo` is not marked `#[upstream]`, we generate:
    /// ```notrust
    /// forall<T> { IsLocal(Foo<T>). }
    /// ```
    ///
    /// Given an `#[upstream]` type that is also fundamental:
    ///
    /// ```notrust
    /// #[upstream]
    /// #[fundamental]
    /// struct Box<T> {}
    /// ```
    ///
    /// We generate the following clauses:
    ///
    /// ```notrust
    /// forall<T> { IsLocal(Box<T>) :- IsLocal(T). }
    ///
    /// forall<T> { IsUpstream(Box<T>) :- IsUpstream(T). }
    ///
    /// // Generated for both upstream and local fundamental types
    /// forall<T> { DownstreamType(Box<T>) :- DownstreamType(T). }
    /// ```
    ///
    fn to_program_clauses(&self, _program: &dyn RustIrSource, clauses: &mut Vec<ProgramClause>) {
        let wf = self
            .binders
            .map_ref(|bound_datum| ProgramClauseImplication {
                consequence: WellFormed::Ty(bound_datum.self_ty.clone().cast()).cast(),

                conditions: bound_datum
                    .where_clauses
                    .iter()
                    .cloned()
                    .map(|wc| wc.map(|bound| bound.into_well_formed_goal()))
                    .casted()
                    .collect(),
            })
            .cast();

        let is_fully_visible = self
            .binders
            .map_ref(|bound_datum| ProgramClauseImplication {
                consequence: DomainGoal::IsFullyVisible(bound_datum.self_ty.clone().cast()),
                conditions: bound_datum
                    .self_ty
                    .type_parameters()
                    .map(|ty| DomainGoal::IsFullyVisible(ty).cast())
                    .collect(),
            })
            .cast();

        clauses.push(wf);
        clauses.push(is_fully_visible);

        // Fundamental types often have rules in the form of:
        //     Goal(FundamentalType<T>) :- Goal(T)
        // This macro makes creating that kind of clause easy
        macro_rules! fundamental_rule {
            ($goal:ident) => {
                // Fundamental types must always have at least one type parameter for this rule to
                // make any sense. We currently do not have have any fundamental types with more than
                // one type parameter, nor do we know what the behaviour for that should be. Thus, we
                // are asserting here that there is only a single type parameter until the day when
                // someone makes a decision about how that should behave.
                assert_eq!(self.binders.value.self_ty.len_type_parameters(), 1,
                    "Only fundamental types with a single parameter are supported");

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

        // Types that are not marked `#[upstream]` satisfy IsLocal(TypeName)
        if !self.binders.value.flags.upstream {
            // `IsLocalTy(Ty)` depends *only* on whether the type is marked #[upstream] and nothing else
            let is_local = self
                .binders
                .map_ref(|bound_datum| ProgramClauseImplication {
                    consequence: DomainGoal::IsLocal(bound_datum.self_ty.clone().cast()),
                    conditions: Vec::new(),
                })
                .cast();

            clauses.push(is_local);
        } else if self.binders.value.flags.fundamental {
            // If a type is `#[upstream]`, but is also `#[fundamental]`, it satisfies IsLocal
            // if and only if its parameters satisfy IsLocal
            fundamental_rule!(IsLocal);
            fundamental_rule!(IsUpstream);
        } else {
            // The type is just upstream and not fundamental

            let is_upstream = self
                .binders
                .map_ref(|bound_datum| ProgramClauseImplication {
                    consequence: DomainGoal::IsUpstream(bound_datum.self_ty.clone().cast()),
                    conditions: Vec::new(),
                })
                .cast();

            clauses.push(is_upstream);
        }

        if self.binders.value.flags.fundamental {
            fundamental_rule!(DownstreamType);
        }

        let condition = DomainGoal::FromEnv(FromEnv::Ty(self.binders.value.self_ty.clone().cast()));

        for wc in self
            .binders
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
            clauses.push(
                Binders {
                    binders: wc
                        .binders
                        .into_iter()
                        .chain(self.binders.binders.clone())
                        .collect(),
                    value: ProgramClauseImplication {
                        consequence: wc.value,
                        conditions: vec![condition.clone().shifted_in(shift).cast()],
                    },
                }
                .cast(),
            );
        }
    }
}

impl ToProgramClauses for TraitDatum {
    /// Given the following trait declaration: `trait Ord<T> where Self: Eq<T> { ... }`, generate:
    ///
    /// ```notrust
    /// -- Rule WellFormed-TraitRef
    /// forall<Self, T> {
    ///    WF(Self: Ord<T>) :- Implemented(Self: Ord<T>), WF(Self: Eq<T>).
    /// }
    /// ```
    ///
    /// and the reverse rules:
    ///
    /// ```notrust
    /// -- Rule Implemented-From-Env
    /// forall<Self, T> {
    ///    (Self: Ord<T>) :- FromEnv(Self: Ord<T>).
    /// }
    ///
    /// -- Rule Implied-Bound-From-Trait
    /// forall<Self, T> {
    ///     FromEnv(Self: Eq<T>) :- FromEnv(Self: Ord<T>).
    /// }
    /// ```
    ///
    /// As specified in the orphan rules, if a trait is not marked `#[upstream]`, the current crate
    /// can implement it for any type. To represent that, we generate:
    ///
    /// ```notrust
    /// // `Ord<T>` would not be `#[upstream]` when compiling `std`
    /// forall<Self, T> { LocalImplAllowed(Self: Ord<T>). }
    /// ```
    ///
    /// For traits that are `#[upstream]` (i.e. not in the current crate), the orphan rules dictate
    /// that impls are allowed as long as at least one type parameter is local and each type
    /// prior to that is fully visible. That means that each type prior to the first local
    /// type cannot contain any of the type parameters of the impl.
    ///
    /// This rule is fairly complex, so we expand it and generate a program clause for each
    /// possible case. This is represented as follows:
    ///
    /// ```notrust
    /// // for `#[upstream] trait Foo<T, U, V> where Self: Eq<T> { ... }`
    /// forall<Self, T, U, V> {
    ///     LocalImplAllowed(Self: Foo<T, U, V>) :- IsLocal(Self).
    /// }
    ///
    /// forall<Self, T, U, V> {
    ///     LocalImplAllowed(Self: Foo<T, U, V>) :-
    ///         IsFullyVisible(Self),
    ///         IsLocal(T).
    /// }
    ///
    /// forall<Self, T, U, V> {
    ///     LocalImplAllowed(Self: Foo<T, U, V>) :-
    ///         IsFullyVisible(Self),
    ///         IsFullyVisible(T),
    ///         IsLocal(U).
    /// }
    ///
    /// forall<Self, T, U, V> {
    ///     LocalImplAllowed(Self: Foo<T, U, V>) :-
    ///         IsFullyVisible(Self),
    ///         IsFullyVisible(T),
    ///         IsFullyVisible(U),
    ///         IsLocal(V).
    /// }
    /// ```
    ///
    /// The overlap check uses compatible { ... } mode to ensure that it accounts for impls that
    /// may exist in some other *compatible* world. For every upstream trait, we add a rule to
    /// account for the fact that upstream crates are able to compatibly add impls of upstream
    /// traits for upstream types.
    ///
    /// ```notrust
    /// // For `#[upstream] trait Foo<T, U, V> where Self: Eq<T> { ... }`
    /// forall<Self, T, U, V> {
    ///     Implemented(Self: Foo<T, U, V>) :-
    ///         Implemented(Self: Eq<T>), // where clauses
    ///         Compatible,               // compatible modality
    ///         IsUpstream(Self),
    ///         IsUpstream(T),
    ///         IsUpstream(U),
    ///         IsUpstream(V),
    ///         CannotProve.              // returns ambiguous
    /// }
    /// ```
    ///
    /// In certain situations, this is too restrictive. Consider the following code:
    ///
    /// ```notrust
    /// /* In crate std */
    /// trait Sized { }
    /// struct str { }
    ///
    /// /* In crate bar (depends on std) */
    /// trait Bar { }
    /// impl Bar for str { }
    /// impl<T> Bar for T where T: Sized { }
    /// ```
    ///
    /// Here, because of the rules we've defined, these two impls overlap. The std crate is
    /// upstream to bar, and thus it is allowed to compatibly implement Sized for str. If str
    /// can implement Sized in a compatible future, these two impls definitely overlap since the
    /// second impl covers all types that implement Sized.
    ///
    /// The solution we've got right now is to mark Sized as "fundamental" when it is defined.
    /// This signals to the Rust compiler that it can rely on the fact that str does not
    /// implement Sized in all contexts. A consequence of this is that we can no longer add an
    /// implementation of Sized compatibly for str. This is the trade off you make when defining
    /// a fundamental trait.
    ///
    /// To implement fundamental traits, we simply just do not add the rule above that allows
    /// upstream types to implement upstream traits. Fundamental traits are not allowed to
    /// compatibly do that.
    fn to_program_clauses(&self, _program: &dyn RustIrSource, clauses: &mut Vec<ProgramClause>) {
        let trait_ref = self.binders.value.trait_ref.clone();

        let trait_ref_impl = WhereClause::Implemented(self.binders.value.trait_ref.clone());

        let wf = self
            .binders
            .map_ref(|bound| ProgramClauseImplication {
                consequence: WellFormed::Trait(trait_ref.clone()).cast(),

                conditions: {
                    bound
                        .where_clauses
                        .iter()
                        .cloned()
                        .map(|wc| wc.map(|bound| bound.into_well_formed_goal()))
                        .casted()
                        .chain(Some(DomainGoal::Holds(trait_ref_impl.clone()).cast()))
                        .collect()
                },
            })
            .cast();

        clauses.push(wf);

        // The number of parameters will always be at least 1 because of the Self parameter
        // that is automatically added to every trait. This is important because otherwise
        // the added program clauses would not have any conditions.
        let type_parameters: Vec<_> = self.binders.value.trait_ref.type_parameters().collect();

        // Add all cases for potential downstream impls that could exist
        clauses.extend((0..type_parameters.len()).map(|i| {
            let impl_may_exist =
                self.binders
                    .map_ref(|bound_datum| ProgramClauseImplication {
                        consequence: DomainGoal::Holds(WhereClause::Implemented(
                            bound_datum.trait_ref.clone(),
                        )),
                        conditions: bound_datum
                            .where_clauses
                            .iter()
                            .cloned()
                            .casted()
                            .chain(iter::once(DomainGoal::Compatible(()).cast()))
                            .chain((0..i).map(|j| {
                                DomainGoal::IsFullyVisible(type_parameters[j].clone()).cast()
                            }))
                            .chain(iter::once(
                                DomainGoal::DownstreamType(type_parameters[i].clone()).cast(),
                            ))
                            .chain(iter::once(Goal::CannotProve(())))
                            .collect(),
                    })
                    .cast();

            impl_may_exist
        }));

        if !self.binders.value.flags.upstream {
            let impl_allowed = self
                .binders
                .map_ref(|bound_datum| ProgramClauseImplication {
                    consequence: DomainGoal::LocalImplAllowed(bound_datum.trait_ref.clone()),
                    conditions: Vec::new(),
                })
                .cast();

            clauses.push(impl_allowed);
        } else {
            clauses.extend((0..type_parameters.len()).map(|i| {
                let impl_maybe_allowed = self
                    .binders
                    .map_ref(|bound_datum| ProgramClauseImplication {
                        consequence: DomainGoal::LocalImplAllowed(bound_datum.trait_ref.clone()),
                        conditions: (0..i)
                            .map(|j| DomainGoal::IsFullyVisible(type_parameters[j].clone()).cast())
                            .chain(iter::once(
                                DomainGoal::IsLocal(type_parameters[i].clone()).cast(),
                            ))
                            .collect(),
                    })
                    .cast();

                impl_maybe_allowed
            }));

            // Fundamental traits can be reasoned about negatively without any ambiguity, so no
            // need for this rule if the trait is fundamental.
            if !self.binders.value.flags.fundamental {
                let impl_may_exist = self
                    .binders
                    .map_ref(|bound_datum| ProgramClauseImplication {
                        consequence: DomainGoal::Holds(WhereClause::Implemented(
                            bound_datum.trait_ref.clone(),
                        )),
                        conditions: bound_datum
                            .where_clauses
                            .iter()
                            .cloned()
                            .casted()
                            .chain(iter::once(DomainGoal::Compatible(()).cast()))
                            .chain(
                                bound_datum
                                    .trait_ref
                                    .type_parameters()
                                    .map(|ty| DomainGoal::IsUpstream(ty).cast()),
                            )
                            .chain(iter::once(Goal::CannotProve(())))
                            .collect(),
                    })
                    .cast();

                clauses.push(impl_may_exist);
            }
        }

        let condition = DomainGoal::FromEnv(FromEnv::Trait(trait_ref.clone()));

        clauses.extend(
            self.binders
                .value
                .where_clauses
                .iter()
                .cloned()
                .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
                .map(|wc| {
                    // We move the binders of the where-clause to the left for the reverse rules,
                    // cf `StructDatum::to_program_clauses`.
                    let shift = wc.binders.len();

                    Binders {
                        binders: wc
                            .binders
                            .into_iter()
                            .chain(self.binders.binders.clone())
                            .collect(),
                        value: ProgramClauseImplication {
                            consequence: wc.value,
                            conditions: vec![condition.clone().shifted_in(shift).cast()],
                        },
                    }
                    .cast()
                }),
        );

        clauses.push(
            self.binders
                .map_ref(|_| ProgramClauseImplication {
                    consequence: DomainGoal::Holds(trait_ref_impl),
                    conditions: vec![condition.cast()],
                })
                .cast(),
        );
    }
}

impl ToProgramClauses for AssociatedTyDatum {
    /// For each associated type, we define the "projection
    /// equality" rules. There are always two; one for a successful normalization,
    /// and one for the "fallback" notion of equality.
    ///
    /// Given: (here, `'a` and `T` represent zero or more parameters)
    ///
    /// ```notrust
    /// trait Foo {
    ///     type Assoc<'a, T>: Bounds where WC;
    /// }
    /// ```
    ///
    /// we generate the 'fallback' rule:
    ///
    /// ```notrust
    /// -- Rule ProjectionEq-Placeholder
    /// forall<Self, 'a, T> {
    ///     ProjectionEq(<Self as Foo>::Assoc<'a, T> = (Foo::Assoc<'a, T>)<Self>).
    /// }
    /// ```
    ///
    /// and
    ///
    /// ```notrust
    /// -- Rule ProjectionEq-Normalize
    /// forall<Self, 'a, T, U> {
    ///     ProjectionEq(<T as Foo>::Assoc<'a, T> = U) :-
    ///         Normalize(<T as Foo>::Assoc -> U).
    /// }
    /// ```
    ///
    /// We used to generate an "elaboration" rule like this:
    ///
    /// ```notrust
    /// forall<T> {
    ///     T: Foo :- exists<U> { ProjectionEq(<T as Foo>::Assoc = U) }.
    /// }
    /// ```
    ///
    /// but this caused problems with the recursive solver. In
    /// particular, whenever normalization is possible, we cannot
    /// solve that projection uniquely, since we can now elaborate
    /// `ProjectionEq` to fallback *or* normalize it. So instead we
    /// handle this kind of reasoning through the `FromEnv` predicate.
    ///
    /// We also generate rules specific to WF requirements and implied bounds:
    ///
    /// ```notrust
    /// -- Rule WellFormed-AssocTy
    /// forall<Self, 'a, T> {
    ///     WellFormed((Foo::Assoc)<Self, 'a, T>) :- WellFormed(Self: Foo), WellFormed(WC).
    /// }
    ///
    /// -- Rule Implied-WC-From-AssocTy
    /// forall<Self, 'a, T> {
    ///     FromEnv(WC) :- FromEnv((Foo::Assoc)<Self, 'a, T>).
    /// }
    ///
    /// -- Rule Implied-Bound-From-AssocTy
    /// forall<Self, 'a, T> {
    ///     FromEnv(<Self as Foo>::Assoc<'a,T>: Bounds) :- FromEnv(Self: Foo), WC.
    /// }
    ///
    /// -- Rule Implied-Trait-From-AssocTy
    /// forall<Self,'a, T> {
    ///     FromEnv(Self: Foo) :- FromEnv((Foo::Assoc)<Self, 'a,T>).
    /// }
    /// ```
    fn to_program_clauses(&self, program: &dyn RustIrSource, clauses: &mut Vec<ProgramClause>) {
        let binders: Vec<_> = self
            .parameter_kinds
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

        // Fallback rule. The solver uses this to move between the projection
        // and placeholder type.
        //
        //    forall<Self> {
        //        ProjectionEq(<Self as Foo>::Assoc = (Foo::Assoc)<Self>).
        //    }
        clauses.push(
            Binders {
                binders: binders.clone(),
                value: ProgramClauseImplication {
                    consequence: projection_eq.clone().cast(),
                    conditions: vec![],
                },
            }
            .cast(),
        );

        // Well-formedness of projection type.
        //
        //    forall<Self> {
        //        WellFormed((Foo::Assoc)<Self>) :- WellFormed(Self: Foo), WellFormed(WC).
        //    }
        clauses.push(
            Binders {
                binders: binders.clone(),
                value: ProgramClauseImplication {
                    consequence: WellFormed::Ty(app_ty.clone()).cast(),
                    conditions: iter::once(WellFormed::Trait(trait_ref.clone()).cast())
                        .chain(
                            self.where_clauses
                                .iter()
                                .cloned()
                                .map(|wc| wc.map(|bound| bound.into_well_formed_goal()))
                                .casted(),
                        )
                        .collect(),
                },
            }
            .cast(),
        );

        // Assuming well-formedness of projection type means we can assume
        // the trait ref as well. Mostly used in function bodies.
        //
        //    forall<Self> {
        //        FromEnv(Self: Foo) :- FromEnv((Foo::Assoc)<Self>).
        //    }
        clauses.push(
            Binders {
                binders: binders.clone(),
                value: ProgramClauseImplication {
                    consequence: FromEnv::Trait(trait_ref.clone()).cast(),
                    conditions: vec![FromEnv::Ty(app_ty.clone()).cast()],
                },
            }
            .cast(),
        );

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
                    conditions: vec![FromEnv::Ty(app_ty.clone()).shifted_in(shift).cast()],
                },
            }
            .cast()
        }));

        // Reverse rule for implied bounds.
        //
        //    forall<Self> {
        //        FromEnv(<Self as Foo>::Assoc: Bounds) :- FromEnv(Self: Foo), WC
        //    }
        clauses.extend(self.bounds_on_self().into_iter().map(|bound| {
            // Same as above in case of higher-ranked inline bounds.
            let shift = bound.binders.len();
            let from_env_trait =
                iter::once(FromEnv::Trait(trait_ref.clone()).shifted_in(shift).cast());

            let where_clauses = self.where_clauses.iter().cloned().casted();

            Binders {
                binders: bound
                    .binders
                    .iter()
                    .chain(binders.iter())
                    .cloned()
                    .collect(),
                value: ProgramClauseImplication {
                    consequence: bound.value.clone().into_from_env_goal(),
                    conditions: from_env_trait.chain(where_clauses).collect(),
                },
            }
            .cast()
        }));

        // add new type parameter U
        let mut binders = binders;
        binders.push(ParameterKind::Ty(()));
        let ty = Ty::BoundVar(binders.len() - 1);

        // `Normalize(<T as Foo>::Assoc -> U)`
        let normalize = Normalize {
            projection: projection.clone(),
            ty: ty.clone(),
        };

        // `ProjectionEq(<T as Foo>::Assoc = U)`
        let projection_eq = ProjectionEq {
            projection: projection.clone(),
            ty,
        };

        // Projection equality rule from above.
        //
        //    forall<T, U> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :-
        //            Normalize(<T as Foo>::Assoc -> U).
        //    }
        clauses.push(
            Binders {
                binders: binders.clone(),
                value: ProgramClauseImplication {
                    consequence: projection_eq.clone().cast(),
                    conditions: vec![normalize.clone().cast()],
                },
            }
            .cast(),
        );
    }
}
