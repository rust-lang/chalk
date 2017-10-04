use cast::Cast;
use fold::{ExistentialFolder, Fold, UniversalFolder};
use std::sync::Arc;
use zip::{Zip, Zipper};

use super::*;
use super::var::*;

impl InferenceTable {
    pub fn unify<T>(&mut self,
                    environment: &Arc<Environment>,
                    a: &T,
                    b: &T)
                    -> Result<UnificationResult>
        where T: ?Sized + Zip,
    {
        debug_heading!("unify(a={:?}\
                     ,\n      b={:?})", a, b);
        let snapshot = self.snapshot();
        match Unifier::new(self, environment).unify(a, b) {
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

struct Unifier<'t> {
    table: &'t mut InferenceTable,
    environment: &'t Arc<Environment>,
    goals: Vec<InEnvironment<DomainGoal>>,
    constraints: Vec<InEnvironment<Constraint>>,
}

#[derive(Debug)]
pub struct UnificationResult {
    pub goals: Vec<InEnvironment<DomainGoal>>,
    pub constraints: Vec<InEnvironment<Constraint>>,
}

impl<'t> Unifier<'t> {
    fn new(table: &'t mut InferenceTable, environment: &'t Arc<Environment>) -> Self {
        Unifier {
            environment: environment,
            table: table,
            goals: vec![],
            constraints: vec![],
        }
    }

    /// The main entry point for the `Unifier` type and really the
    /// only type meant to be called externally. Performs a
    /// unification of `a` and `b` and returns the Unification Result.
    fn unify<T>(mut self, a: &T, b: &T) -> Result<UnificationResult>
        where T: ?Sized + Zip,
    {
        Zip::zip_with(&mut self, a, b)?;
        Ok(UnificationResult {
            goals: self.goals,
            constraints: self.constraints,
        })
    }

    /// When we encounter a "sub-unification" problem that is in a distinct
    /// environment, we invoke this routine.
    fn sub_unify<T>(&mut self,
                    environment: &Arc<Environment>,
                    ty1: T,
                    ty2: T)
                    -> Result<()>
        where T: Zip + Fold,
    {
        let sub_unifier = Unifier::new(self.table, environment);
        let UnificationResult { goals, constraints } = sub_unifier.unify(&ty1, &ty2)?;
        self.goals.extend(goals);
        self.constraints.extend(constraints);
        Ok(())
    }

    fn unify_ty_ty<'a>(&mut self, a: &'a Ty, b: &'a Ty) -> Result<()> {
        //         ^^                 ^^         ^^ FIXME rustc bug
        if let Some(n_a) = self.table.normalize_shallow(a) {
            return self.unify_ty_ty(&n_a, b);
        } else if let Some(n_b) = self.table.normalize_shallow(b) {
            return self.unify_ty_ty(a, &n_b);
        }

        debug_heading!("unify_ty_ty(a={:?}\
                     ,\n            b={:?})", a, b);

        match (a, b) {
            (&Ty::Var(depth1), &Ty::Var(depth2)) => {
                let var1 = TyInferenceVariable::from_depth(depth1);
                let var2 = TyInferenceVariable::from_depth(depth2);
                debug!("unify_ty_ty: unify_var_var({:?}, {:?})", var1, var2);
                Ok(self.table
                    .ty_unify
                    .unify_var_var(var1, var2)
                    .expect("unification of two unbound variables cannot fail"))
            }

            (&Ty::Var(depth), ty @ &Ty::Apply(_)) |
            (ty @ &Ty::Apply(_), &Ty::Var(depth)) |
            (&Ty::Var(depth), ty @ &Ty::ForAll(_)) |
            (ty @ &Ty::ForAll(_), &Ty::Var(depth)) => {
                self.unify_var_ty(TyInferenceVariable::from_depth(depth), ty)
            }

            (&Ty::ForAll(ref quantified_ty1), &Ty::ForAll(ref quantified_ty2)) => {
                self.unify_forall_tys(quantified_ty1, quantified_ty2)
            }

            (&Ty::ForAll(ref quantified_ty), apply_ty @ &Ty::Apply(_)) |
            (apply_ty @ &Ty::Apply(_), &Ty::ForAll(ref quantified_ty)) => {
                self.unify_forall_apply(quantified_ty, apply_ty)
            }

            (&Ty::Apply(ref apply1), &Ty::Apply(ref apply2)) => {
                if apply1.name != apply2.name {
                    bail!("cannot equate `{:?}` and `{:?}`", apply1.name, apply2.name);
                }

                Zip::zip_with(self, &apply1.parameters, &apply2.parameters)
            }

            (&Ty::Projection(ref proj1), &Ty::Projection(ref proj2)) => {
                self.unify_projection_tys(proj1, proj2)
            }

            (ty @ &Ty::Apply(_), &Ty::Projection(ref proj)) |
            (ty @ &Ty::ForAll(_), &Ty::Projection(ref proj)) |
            (ty @ &Ty::Var(_), &Ty::Projection(ref proj)) |
            (&Ty::Projection(ref proj), ty @ &Ty::Apply(_)) |
            (&Ty::Projection(ref proj), ty @ &Ty::ForAll(_)) |
            (&Ty::Projection(ref proj), ty @ &Ty::Var(_)) => {
                self.unify_projection_ty(proj, ty)
            }
        }
    }

    fn unify_forall_tys(&mut self, ty1: &QuantifiedTy, ty2: &QuantifiedTy) -> Result<()> {
        // for<'a...> T == for<'b...> U where 'a != 'b
        //
        // if:
        //
        // for<'a...> exists<'b...> T == U &&
        // for<'b...> exists<'a...> T == U

        debug!("unify_forall_tys({:?}, {:?})", ty1, ty2);

        let mut environment = self.environment.clone();
        let lifetimes1: Vec<_> = (0..ty1.num_binders)
            .map(|_| {
                environment = environment.new_universe();
                Lifetime::ForAll(environment.universe).cast()
            })
            .collect();

        let lifetimes2: Vec<_> = (0..ty2.num_binders)
            .map(|_| self.table.new_lifetime_variable(environment.universe).to_lifetime().cast())
            .collect();

        let ty1 = ty1.subst(&lifetimes1);
        let ty2 = ty2.subst(&lifetimes2);
        debug!("unify_forall_tys: ty1 = {:?}", ty1);
        debug!("unify_forall_tys: ty2 = {:?}", ty2);

        self.sub_unify(&environment, ty1, ty2)
    }

    fn unify_projection_tys(&mut self, proj1: &ProjectionTy, proj2: &ProjectionTy) -> Result<()> {
        let var = self.table.new_variable(self.environment.universe).to_ty();
        self.unify_projection_ty(proj1, &var)?;
        self.unify_projection_ty(proj2, &var)?;
        Ok(())
    }

    fn unify_projection_ty(&mut self, proj: &ProjectionTy, ty: &Ty) -> Result<()> {
        Ok(self.goals.push(InEnvironment::new(self.environment,
                                              Normalize {
                                                  projection: proj.clone(),
                                                  ty: ty.clone(),
                                              }
                                              .cast())))
    }

    fn unify_forall_apply(&mut self, ty1: &QuantifiedTy, ty2: &Ty) -> Result<()> {
        let mut environment = self.environment.clone();
        let lifetimes1: Vec<_> = (0..ty1.num_binders)
            .map(|_| {
                environment = environment.new_universe();
                Lifetime::ForAll(environment.universe).cast()
            })
            .collect();

        let ty1 = ty1.subst(&lifetimes1);
        let ty2 = ty2.clone();

        self.sub_unify(&environment, ty1, ty2)
    }

    fn unify_var_ty(&mut self, var: TyInferenceVariable, ty: &Ty) -> Result<()> {
        debug!("unify_var_ty(var={:?}, ty={:?})", var, ty);

        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = match self.table.ty_unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("`unify_var_apply` invoked on bound var"),
        };

        let ty1 = ty.fold_with(&mut OccursCheck::new(self, var, universe_index), 0)?;

        self.table.ty_unify.unify_var_value(var, InferenceValue::Bound(ty1.clone())).unwrap();
        debug!("unify_var_ty: var {:?} set to {:?}", var, ty1);

        Ok(())
    }

    fn unify_lifetime_lifetime(&mut self, a: &Lifetime, b: &Lifetime) -> Result<()> {
        if let Some(n_a) = self.table.normalize_lifetime(a) {
            return self.unify_lifetime_lifetime(&n_a, b);
        } else if let Some(n_b) = self.table.normalize_lifetime(b) {
            return self.unify_lifetime_lifetime(a, &n_b);
        }

        debug!("unify_lifetime_lifetime({:?}, {:?})", a, b);

        match (a, b) {
            (&Lifetime::Var(depth_a), &Lifetime::Var(depth_b)) => {
                let var_a = LifetimeInferenceVariable::from_depth(depth_a);
                let var_b = LifetimeInferenceVariable::from_depth(depth_b);
                debug!("unify_lifetime_lifetime: var_a={:?} var_b={:?}", var_a, var_b);
                self.table.lifetime_unify.unify_var_var(var_a, var_b).unwrap();
                Ok(())
            }

            (&Lifetime::Var(depth), &Lifetime::ForAll(ui)) |
            (&Lifetime::ForAll(ui), &Lifetime::Var(depth)) => {
                let var = LifetimeInferenceVariable::from_depth(depth);
                let var_ui = match self.table.lifetime_unify.probe_value(var) {
                    InferenceValue::Unbound(ui) => ui,
                    InferenceValue::Bound(_) => panic!("bound var survived normalization"),
                };
                if var_ui.can_see(ui) {
                    let v = Lifetime::ForAll(ui);
                    self.table.lifetime_unify.unify_var_value(var, InferenceValue::Bound(v))
                                             .unwrap();
                    Ok(())
                } else {
                    Ok(self.push_lifetime_eq_constraint(*a, *b))
                }
            }

            (&Lifetime::ForAll(_), &Lifetime::ForAll(_)) => {
                if a != b {
                    Ok(self.push_lifetime_eq_constraint(*a, *b))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn push_lifetime_eq_constraint(&mut self, a: Lifetime, b: Lifetime) {
        self.constraints.push(InEnvironment::new(self.environment, Constraint::LifetimeEq(a, b)));
    }
}

impl<'t> Zipper for Unifier<'t> {
    fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Result<()> {
        self.unify_ty_ty(a, b)
    }

    fn zip_lifetimes(&mut self, a: &Lifetime, b: &Lifetime) -> Result<()> {
        self.unify_lifetime_lifetime(a, b)
    }

    fn zip_binders<T>(&mut self, a: &Binders<T>, b: &Binders<T>) -> Result<()>
        where T: Zip + Fold<Result = T>
    {
        {
            let a = a.instantiate_universally(self.environment);
            let b = self.table.instantiate_binders_in(a.environment.universe, b);
            let () = self.sub_unify(&a.environment, &a.goal, &b)?;
        }

        {
            let b = b.instantiate_universally(self.environment);
            let a = self.table.instantiate_binders_in(b.environment.universe, a);
            let () = self.sub_unify(&b.environment, &a, &b.goal)?;
        }

        Ok(())
    }
}

impl ApplicationTy {
    fn universe_index(&self) -> UniverseIndex {
        self.name.universe_index()
    }
}

impl TypeName {
    fn universe_index(&self) -> UniverseIndex {
        match *self {
            TypeName::ItemId(_) |
            TypeName::AssociatedType(_) => UniverseIndex::root(),
            TypeName::ForAll(universe) => {
                assert!(universe.counter > 0);
                universe
            }
        }
    }
}

struct OccursCheck<'u, 't: 'u> {
    unifier: &'u mut Unifier<'t>,
    var: TyInferenceVariable,
    universe_index: UniverseIndex,
}

impl<'u, 't> OccursCheck<'u, 't> {
    fn new(unifier: &'u mut Unifier<'t>,
           var: TyInferenceVariable,
           universe_index: UniverseIndex)
           -> Self {
        OccursCheck { unifier, var, universe_index }
    }
}

impl<'u, 't> UniversalFolder for OccursCheck<'u, 't> {
    fn fold_free_universal_ty(&mut self,
                              universe: UniverseIndex,
                              _binders: usize)
                              -> Result<Ty> {
        if self.universe_index < universe {
            bail!("incompatible universes(universe_index={:?}, application_universe_index={:?})",
                  self.universe_index, universe)
        } else {
            Ok(TypeName::ForAll(universe).to_ty()) // no need to shift, not relative to depth
        }
    }

    fn fold_free_universal_lifetime(&mut self,
                                    ui: UniverseIndex,
                                    binders: usize)
                                    -> Result<Lifetime> {
        if self.universe_index < ui {
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

            let tick_x = self.unifier.table.new_lifetime_variable(self.universe_index);
            self.unifier.push_lifetime_eq_constraint(tick_x.to_lifetime(), ui.to_lifetime());
            Ok(tick_x.to_lifetime().up_shift(binders))
        } else {
            // If the `ui` is higher than `self.universe_index`, then we can name
            // this lifetime, no problem.
            Ok(ui.to_lifetime()) // no need to shift, not relative to depth
        }
    }
}

impl<'u, 't> ExistentialFolder for OccursCheck<'u, 't> {
    fn fold_free_existential_ty(&mut self, depth: usize, binders: usize) -> Result<Ty> {
        let v = TyInferenceVariable::from_depth(depth - binders);
        match self.unifier.table.ty_unify.probe_value(v) {
            // If this variable already has a value, fold over that value instead.
            InferenceValue::Bound(normalized_ty) => {
                Ok(normalized_ty.fold_with(self, 0)?.up_shift(binders))
            }

            // Otherwise, check the universe of the variable, and also
            // check for cycles with `self.var` (which this will soon
            // become the value of).
            InferenceValue::Unbound(ui) => {
                if self.unifier.table.ty_unify.unioned(v, self.var) {
                    bail!("cycle during unification");
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
                        .ty_unify
                        .unify_var_value(v, InferenceValue::Unbound(self.universe_index))
                        .unwrap();
                }

                Ok(Ty::Var(depth)) // depth already includes binders
            }
        }
    }

    fn fold_free_existential_lifetime(&mut self, depth: usize, binders: usize) -> Result<Lifetime> {
        // a free existentially bound region; find the
        // inference variable it corresponds to
        let v = LifetimeInferenceVariable::from_depth(depth - binders);
        match self.unifier.table.lifetime_unify.probe_value(v) {
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
                        .lifetime_unify
                        .unify_var_value(v, InferenceValue::Unbound(self.universe_index))
                        .unwrap();
                }
                Ok(Lifetime::Var(depth).up_shift(binders))
            }

            InferenceValue::Bound(l) => {
                Ok(l.up_shift(binders))
            }
        }
    }
}
