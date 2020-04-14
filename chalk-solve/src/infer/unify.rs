use super::var::*;
use super::*;
use crate::infer::instantiate::IntoBindersAndValue;
use chalk_engine::fallible::*;
use chalk_ir::cast::Cast;
use chalk_ir::fold::{Fold, Folder};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::zip::{Zip, Zipper};
use std::fmt::Debug;

impl<I: Interner> InferenceTable<I> {
    pub(crate) fn unify<T>(
        &mut self,
        interner: &I,
        environment: &Environment<I>,
        a: &T,
        b: &T,
    ) -> Fallible<UnificationResult<I>>
    where
        T: ?Sized + Zip<I>,
    {
        debug_heading!(
            "unify(a={:?}\
             ,\n      b={:?})",
            a,
            b
        );
        let snapshot = self.snapshot();
        match Unifier::new(interner, self, environment).unify(a, b) {
            Ok(r) => {
                self.commit(snapshot);
                Ok(r)
            }
            Err(e) => {
                self.rollback_to(snapshot);
                Err(e)
            }
        }
    }
}

struct Unifier<'t, I: Interner> {
    table: &'t mut InferenceTable<I>,
    environment: &'t Environment<I>,
    goals: Vec<InEnvironment<DomainGoal<I>>>,
    constraints: Vec<InEnvironment<Constraint<I>>>,
    interner: &'t I,
}

#[derive(Debug)]
pub(crate) struct UnificationResult<I: Interner> {
    pub(crate) goals: Vec<InEnvironment<DomainGoal<I>>>,
    pub(crate) constraints: Vec<InEnvironment<Constraint<I>>>,
}

impl<'t, I: Interner> Unifier<'t, I> {
    fn new(
        interner: &'t I,
        table: &'t mut InferenceTable<I>,
        environment: &'t Environment<I>,
    ) -> Self {
        Unifier {
            environment: environment,
            table: table,
            goals: vec![],
            constraints: vec![],
            interner,
        }
    }

    /// The main entry point for the `Unifier` type and really the
    /// only type meant to be called externally. Performs a
    /// unification of `a` and `b` and returns the Unification Result.
    fn unify<T>(mut self, a: &T, b: &T) -> Fallible<UnificationResult<I>>
    where
        T: ?Sized + Zip<I>,
    {
        Zip::zip_with(&mut self, a, b)?;
        Ok(UnificationResult {
            goals: self.goals,
            constraints: self.constraints,
        })
    }

    fn unify_ty_ty<'a>(&mut self, a: &'a Ty<I>, b: &'a Ty<I>) -> Fallible<()> {
        let interner = self.interner;
        //         ^^                 ^^         ^^ FIXME rustc bug
        if let Some(n_a) = self.table.normalize_shallow(interner, a) {
            return self.unify_ty_ty(&n_a, b);
        } else if let Some(n_b) = self.table.normalize_shallow(interner, b) {
            return self.unify_ty_ty(a, &n_b);
        }

        debug_heading!(
            "unify_ty_ty(a={:?}\
             ,\n            b={:?})",
            a,
            b
        );

        match (a.data(interner), b.data(interner)) {
            // Unifying two inference variables: unify them in the underlying
            // ena table.
            (&TyData::InferenceVar(var1), &TyData::InferenceVar(var2)) => {
                debug!("unify_ty_ty: unify_var_var({:?}, {:?})", var1, var2);
                let var1 = EnaVariable::from(var1);
                let var2 = EnaVariable::from(var2);
                Ok(self
                    .table
                    .unify
                    .unify_var_var(var1, var2)
                    .expect("unification of two unbound variables cannot fail"))
            }

            // Unifying an inference variables with a non-inference variable.
            (&TyData::InferenceVar(var), &TyData::Apply(_))
            | (&TyData::InferenceVar(var), &TyData::Placeholder(_))
            | (&TyData::InferenceVar(var), &TyData::Dyn(_))
            | (&TyData::InferenceVar(var), &TyData::Function(_)) => self.unify_var_ty(var, b),

            (&TyData::Apply(_), &TyData::InferenceVar(var))
            | (&TyData::Placeholder(_), &TyData::InferenceVar(var))
            | (&TyData::Dyn(_), &TyData::InferenceVar(var))
            | (&TyData::Function(_), &TyData::InferenceVar(var)) => self.unify_var_ty(var, a),

            // Unifying `forall<X> { T }` with some other forall type `forall<X> { U }`
            (&TyData::Function(ref fn1), &TyData::Function(ref fn2)) => {
                self.unify_binders(fn1, fn2)
            }

            // This would correspond to unifying a `fn` type with a non-fn
            // type in Rust; error.
            (&TyData::Function(_), &TyData::Apply(_))
            | (&TyData::Function(_), &TyData::Dyn(_))
            | (&TyData::Function(_), &TyData::Placeholder(_))
            | (&TyData::Apply(_), &TyData::Function(_))
            | (&TyData::Placeholder(_), &TyData::Function(_))
            | (&TyData::Dyn(_), &TyData::Function(_)) => {
                return Err(NoSolution);
            }

            (&TyData::Placeholder(ref p1), &TyData::Placeholder(ref p2)) => {
                Zip::zip_with(self, p1, p2)
            }

            (&TyData::Apply(ref apply1), &TyData::Apply(ref apply2)) => {
                Zip::zip_with(self, apply1, apply2)
            }

            // Cannot unify (e.g.) some struct type `Foo` and a placeholder like `T`
            (&TyData::Apply(_), &TyData::Placeholder(_))
            | (&TyData::Placeholder(_), &TyData::Apply(_)) => {
                return Err(NoSolution);
            }

            // Cannot unify `dyn Trait` with things like structs or placeholders
            (&TyData::Placeholder(_), &TyData::Dyn(_))
            | (&TyData::Dyn(_), &TyData::Placeholder(_))
            | (&TyData::Apply(_), &TyData::Dyn(_))
            | (&TyData::Dyn(_), &TyData::Apply(_)) => {
                return Err(NoSolution);
            }

            // Unifying two dyn is possible if they have the same bounds.
            (&TyData::Dyn(ref qwc1), &TyData::Dyn(ref qwc2)) => Zip::zip_with(self, qwc1, qwc2),

            // Unifying an alias type with some other type `U`.
            (&TyData::Apply(_), &TyData::Alias(ref alias))
            | (&TyData::Placeholder(_), &TyData::Alias(ref alias))
            | (&TyData::Function(_), &TyData::Alias(ref alias))
            | (&TyData::InferenceVar(_), &TyData::Alias(ref alias))
            | (&TyData::Dyn(_), &TyData::Alias(ref alias)) => self.unify_alias_ty(alias, a),

            (&TyData::Alias(ref alias), &TyData::Alias(_))
            | (&TyData::Alias(ref alias), &TyData::Apply(_))
            | (&TyData::Alias(ref alias), &TyData::Placeholder(_))
            | (&TyData::Alias(ref alias), &TyData::Function(_))
            | (&TyData::Alias(ref alias), &TyData::InferenceVar(_))
            | (&TyData::Alias(ref alias), &TyData::Dyn(_)) => self.unify_alias_ty(alias, b),

            (TyData::BoundVar(_), _) | (_, TyData::BoundVar(_)) => panic!(
                "unification encountered bound variable: a={:?} b={:?}",
                a, b
            ),
        }
    }

    fn unify_binders<'a, T, R>(
        &mut self,
        a: impl IntoBindersAndValue<'a, I, Value = T> + Copy + Debug,
        b: impl IntoBindersAndValue<'a, I, Value = T> + Copy + Debug,
    ) -> Fallible<()>
    where
        T: Fold<I, Result = R>,
        R: Zip<I> + Fold<I, Result = R>,
        't: 'a,
    {
        // for<'a...> T == for<'b...> U
        //
        // if:
        //
        // for<'a...> exists<'b...> T == U &&
        // for<'b...> exists<'a...> T == U

        debug!("unify_binders({:?}, {:?})", a, b);
        let interner = self.interner;

        {
            let a_universal = self.table.instantiate_binders_universally(interner, a);
            let b_existential = self.table.instantiate_binders_existentially(interner, b);
            Zip::zip_with(self, &a_universal, &b_existential)?;
        }

        {
            let b_universal = self.table.instantiate_binders_universally(interner, b);
            let a_existential = self.table.instantiate_binders_existentially(interner, a);
            Zip::zip_with(self, &a_existential, &b_universal)
        }
    }

    /// Unify an alias like `<T as Trait>::Item` or `impl Trait` with some other
    /// type `ty` (which might also be an alias). Creates a goal like
    ///
    /// ```notrust
    /// AliasEq(<T as Trait>::Item = U) // associated type projection
    /// AliasEq(impl Trait = U) // impl trait
    /// ```
    fn unify_alias_ty(&mut self, alias: &AliasTy<I>, ty: &Ty<I>) -> Fallible<()> {
        let interner = self.interner;
        Ok(self.goals.push(InEnvironment::new(
            self.environment,
            AliasEq {
                alias: alias.clone(),
                ty: ty.clone(),
            }
            .cast(interner),
        )))
    }

    /// Unify an inference variable `var` with some non-inference
    /// variable `ty`, just bind `var` to `ty`. But we must enforce two conditions:
    ///
    /// - `var` does not appear inside of `ty` (the standard `OccursCheck`)
    /// - `ty` does not reference anything in a lifetime that could not be named in `var`
    ///   (the extended `OccursCheck` created to handle universes)
    fn unify_var_ty(&mut self, var: InferenceVar, ty: &Ty<I>) -> Fallible<()> {
        debug!("unify_var_ty(var={:?}, ty={:?})", var, ty);

        let interner = self.interner;
        let var = EnaVariable::from(var);

        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = self.table.universe_of_unbound_var(var);

        let ty1 = ty.fold_with(
            &mut OccursCheck::new(self, var, universe_index),
            DebruijnIndex::INNERMOST,
        )?;

        self.table
            .unify
            .unify_var_value(var, InferenceValue::from_ty(interner, ty1.clone()))
            .unwrap();
        debug!("unify_var_ty: var {:?} set to {:?}", var, ty1);

        Ok(())
    }

    fn unify_lifetime_lifetime(&mut self, a: &Lifetime<I>, b: &Lifetime<I>) -> Fallible<()> {
        let interner = self.interner;

        if let Some(n_a) = self.table.normalize_lifetime(interner, a) {
            return self.unify_lifetime_lifetime(&n_a, b);
        } else if let Some(n_b) = self.table.normalize_lifetime(interner, b) {
            return self.unify_lifetime_lifetime(a, &n_b);
        }

        debug_heading!("unify_lifetime_lifetime({:?}, {:?})", a, b);

        match (a.data(interner), b.data(interner)) {
            (&LifetimeData::InferenceVar(var_a), &LifetimeData::InferenceVar(var_b)) => {
                let var_a = EnaVariable::from(var_a);
                let var_b = EnaVariable::from(var_b);
                debug!(
                    "unify_lifetime_lifetime: var_a={:?} var_b={:?}",
                    var_a, var_b
                );
                self.table.unify.unify_var_var(var_a, var_b).unwrap();
                Ok(())
            }

            (&LifetimeData::InferenceVar(var), &LifetimeData::Placeholder(idx))
            | (&LifetimeData::Placeholder(idx), &LifetimeData::InferenceVar(var)) => {
                let var = EnaVariable::from(var);
                let var_ui = self.table.universe_of_unbound_var(var);
                if var_ui.can_see(idx.ui) {
                    debug!(
                        "unify_lifetime_lifetime: {:?} in {:?} can see {:?}; unifying",
                        var, var_ui, idx.ui
                    );
                    let v = LifetimeData::Placeholder(idx).intern(interner);
                    self.table
                        .unify
                        .unify_var_value(var, InferenceValue::from_lifetime(interner, v))
                        .unwrap();
                    Ok(())
                } else {
                    debug!(
                        "unify_lifetime_lifetime: {:?} in {:?} cannot see {:?}; pushing constraint",
                        var, var_ui, idx.ui
                    );
                    Ok(self.push_lifetime_eq_constraint(a.clone(), b.clone()))
                }
            }

            (&LifetimeData::Placeholder(_), &LifetimeData::Placeholder(_)) => {
                if a != b {
                    Ok(self.push_lifetime_eq_constraint(a.clone(), b.clone()))
                } else {
                    Ok(())
                }
            }

            (LifetimeData::BoundVar(_), _) | (_, LifetimeData::BoundVar(_)) => panic!(
                "unification encountered bound variable: a={:?} b={:?}",
                a, b
            ),

            (LifetimeData::Phantom(..), _) | (_, LifetimeData::Phantom(..)) => unreachable!(),
        }
    }

    fn push_lifetime_eq_constraint(&mut self, a: Lifetime<I>, b: Lifetime<I>) {
        self.constraints.push(InEnvironment::new(
            self.environment,
            Constraint::LifetimeEq(a, b),
        ));
    }
}

impl<'i, I: Interner> Zipper<'i, I> for Unifier<'i, I> {
    fn zip_tys(&mut self, a: &Ty<I>, b: &Ty<I>) -> Fallible<()> {
        self.unify_ty_ty(a, b)
    }

    fn zip_lifetimes(&mut self, a: &Lifetime<I>, b: &Lifetime<I>) -> Fallible<()> {
        self.unify_lifetime_lifetime(a, b)
    }

    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Fallible<()>
    where
        T: HasInterner<Interner = I> + Zip<I> + Fold<I, Result = T>,
    {
        // The binders that appear in types (apart from quantified types, which are
        // handled in `unify_ty`) appear as part of `dyn Trait` and `impl Trait` types.
        //
        // They come in two varieties:
        //
        // * The existential binder from `dyn Trait` / `impl Trait`
        //   (representing the hidden "self" type)
        // * The `for<..>` binders from higher-ranked traits.
        //
        // In both cases we can use the same `unify_binders` routine.

        self.unify_binders(a, b)
    }

    fn interner(&self) -> &'i I {
        self.interner
    }
}

struct OccursCheck<'u, 't, I: Interner> {
    unifier: &'u mut Unifier<'t, I>,
    var: EnaVariable<I>,
    universe_index: UniverseIndex,
}

impl<'u, 't, I: Interner> OccursCheck<'u, 't, I> {
    fn new(
        unifier: &'u mut Unifier<'t, I>,
        var: EnaVariable<I>,
        universe_index: UniverseIndex,
    ) -> Self {
        OccursCheck {
            unifier,
            var,
            universe_index,
        }
    }
}

impl<'i, I: Interner> Folder<'i, I> for OccursCheck<'_, 'i, I>
where
    I: 'i,
{
    fn as_dyn(&mut self) -> &mut dyn Folder<'i, I> {
        self
    }

    fn fold_free_placeholder_ty(
        &mut self,
        universe: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        let interner = self.interner();
        if self.universe_index < universe.ui {
            Err(NoSolution)
        } else {
            Ok(universe.to_ty(interner)) // no need to shift, not relative to depth
        }
    }

    fn fold_free_placeholder_lifetime(
        &mut self,
        ui: PlaceholderIndex,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        let interner = self.interner();
        if self.universe_index < ui.ui {
            // Scenario is like:
            //
            // exists<T> forall<'b> ?T = Foo<'b>
            //
            // unlike with a type variable, this **might** be
            // ok.  Ultimately it depends on whether the
            // `forall` also introduced relations to lifetimes
            // nameable in T. To handle that, we introduce a
            // fresh region variable `'x` in same universe as `T`
            // and add a side-constraint that `'x = 'b`:
            //
            // exists<'x> forall<'b> ?T = Foo<'x>, where 'x = 'b

            let tick_x = self.unifier.table.new_variable(self.universe_index);
            self.unifier.push_lifetime_eq_constraint(
                tick_x.to_lifetime(interner),
                ui.to_lifetime(interner),
            );
            Ok(tick_x.to_lifetime(interner))
        } else {
            // If the `ui` is higher than `self.universe_index`, then we can name
            // this lifetime, no problem.
            Ok(ui.to_lifetime(interner)) // no need to shift, not relative to depth
        }
    }

    fn fold_inference_ty(
        &mut self,
        var: InferenceVar,
        _outer_binder: DebruijnIndex,
    ) -> Fallible<Ty<I>> {
        let interner = self.interner();
        let var = EnaVariable::from(var);
        match self.unifier.table.unify.probe_value(var) {
            // If this variable already has a value, fold over that value instead.
            InferenceValue::Bound(normalized_ty) => {
                let normalized_ty = normalized_ty.ty(interner).unwrap();
                let normalized_ty = normalized_ty.fold_with(self, DebruijnIndex::INNERMOST)?;
                assert!(!normalized_ty.needs_shift(interner));
                Ok(normalized_ty)
            }

            // Otherwise, check the universe of the variable, and also
            // check for cycles with `self.var` (which this will soon
            // become the value of).
            InferenceValue::Unbound(ui) => {
                if self.unifier.table.unify.unioned(var, self.var) {
                    return Err(NoSolution);
                }

                if self.universe_index < ui {
                    // Scenario is like:
                    //
                    // ?A = foo(?B)
                    //
                    // where ?A is in universe 0 and ?B is in universe 1.
                    // This is OK, if ?B is promoted to universe 0.
                    self.unifier
                        .table
                        .unify
                        .unify_var_value(var, InferenceValue::Unbound(self.universe_index))
                        .unwrap();
                }

                Ok(var.to_ty(interner))
            }
        }
    }

    fn fold_inference_lifetime(
        &mut self,
        var: InferenceVar,
        outer_binder: DebruijnIndex,
    ) -> Fallible<Lifetime<I>> {
        // a free existentially bound region; find the
        // inference variable it corresponds to
        let interner = self.interner();
        let var = EnaVariable::from(var);
        match self.unifier.table.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => {
                if self.universe_index < ui {
                    // Scenario is like:
                    //
                    // exists<T> forall<'b> exists<'a> ?T = Foo<'a>
                    //
                    // where ?A is in universe 0 and `'b` is in universe 1.
                    // This is OK, if `'b` is promoted to universe 0.
                    self.unifier
                        .table
                        .unify
                        .unify_var_value(var, InferenceValue::Unbound(self.universe_index))
                        .unwrap();
                }
                Ok(var.to_lifetime(interner))
            }

            InferenceValue::Bound(l) => {
                let l = l.lifetime(interner).unwrap();
                let l = l.fold_with(self, outer_binder)?;
                assert!(!l.needs_shift(interner));
                Ok(l.clone())
            }
        }
    }

    fn forbid_free_vars(&self) -> bool {
        true
    }

    fn interner(&self) -> &'i I {
        self.unifier.interner
    }

    fn target_interner(&self) -> &'i I {
        self.interner()
    }
}
