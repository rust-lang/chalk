use cast::Cast;
use ir::*;
use std::fmt::Debug;
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
        where T: ?Sized + Zip + Debug,
    {
        debug_heading!("unify(a={:?}\
                     ,\n      b={:?})", a, b);
        let mut unifier = Unifier::new(self, environment);
        match Zip::zip_with(&mut unifier, a, b) {
            Ok(()) => unifier.commit(),
            Err(e) => {
                unifier.rollback();
                Err(e)
            }
        }
    }
}

struct Unifier<'t> {
    table: &'t mut InferenceTable,
    environment: &'t Arc<Environment>,
    snapshot: InferenceSnapshot,
    goals: Vec<InEnvironment<LeafGoal>>,
    constraints: Vec<InEnvironment<Constraint>>,
    cannot_prove: bool,
}

#[derive(Debug)]
pub struct UnificationResult {
    pub goals: Vec<InEnvironment<LeafGoal>>,
    pub constraints: Vec<InEnvironment<Constraint>>,

    /// When unifying two skolemized (forall-quantified) type names, we can
    /// neither confirm nor deny their equality, since we interpret the
    /// unification request as talking about *all possible
    /// substitutions*. Instead, we return an ambiguous result.
    pub cannot_prove: bool,
}

impl<'t> Unifier<'t> {
    fn new(table: &'t mut InferenceTable, environment: &'t Arc<Environment>) -> Self {
        let snapshot = table.snapshot();
        Unifier {
            environment: environment,
            table: table,
            snapshot: snapshot,
            goals: vec![],
            constraints: vec![],
            cannot_prove: false,
        }
    }

    fn commit(self) -> Result<UnificationResult> {
        self.table.commit(self.snapshot);
        Ok(UnificationResult {
            goals: self.goals,
            constraints: self.constraints,
            cannot_prove: self.cannot_prove,
        })
    }

    fn rollback(self) {
        self.table.rollback_to(self.snapshot);
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
                    if apply1.name.is_for_all() || apply2.name.is_for_all() {
                        // we're being asked to prove something like `!0 = !1`
                        // or `!0 = i32`. We interpret this as being asked
                        // whether that holds *for all subtitutions*. Thus, we
                        // cannot prove the goal. That means we get:
                        //
                        //     forall<T, U> { T = U } // CannotProve
                        //     forall<T, U> { not { T = U } } // CannotProve

                        self.cannot_prove = true;
                        return Ok(())
                    } else {
                        bail!("cannot equate `{:?}` and `{:?}`", apply1.name, apply2.name);
                    }
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

        let eq_goal = EqGoal { a: ParameterKind::Ty(ty1), b: ParameterKind::Ty(ty2) };
        let goal = InEnvironment::new(&environment, eq_goal).cast();
        debug!("unify_forall_tys: goal = {:?}", goal);

        self.goals.push(goal);

        Ok(())
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

        let eq_goal = EqGoal { a: ParameterKind::Ty(ty1), b: ParameterKind::Ty(ty2) };
        self.goals.push(InEnvironment::new(&environment, eq_goal).cast());

        Ok(())
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

        let ty1 = OccursCheck::new(self, var, universe_index).check_ty(ty)?;

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
    binders: usize,
    var: TyInferenceVariable,
    universe_index: UniverseIndex,
}

impl<'u, 't> OccursCheck<'u, 't> {
    fn new(unifier: &'u mut Unifier<'t>,
           var: TyInferenceVariable,
           universe_index: UniverseIndex)
           -> Self {
        OccursCheck { unifier, binders: 0, var, universe_index }
    }

    fn check_apply(&mut self, apply: &ApplicationTy) -> Result<ApplicationTy> {
        let ApplicationTy { name, ref parameters } = *apply;
        self.universe_check(name.universe_index())?;
        let parameters = parameters.iter()
                                   .map(|p| self.check_parameter(p))
                                   .collect::<Result<Vec<_>>>()?;
        Ok(ApplicationTy { name, parameters })
    }

    fn check_quantified(&mut self, quantified_ty: &QuantifiedTy) -> Result<QuantifiedTy> {
        let QuantifiedTy { num_binders, ref ty } = *quantified_ty;
        self.binders += num_binders;
        let ty = self.check_ty(ty)?;
        self.binders -= num_binders;
        Ok(QuantifiedTy { num_binders, ty })
    }

    fn check_parameter(&mut self, arg: &Parameter) -> Result<Parameter> {
        match *arg {
            ParameterKind::Ty(ref t) => Ok(ParameterKind::Ty(self.check_ty(t)?)),
            ParameterKind::Lifetime(ref lt) => Ok(ParameterKind::Lifetime(self.check_lifetime(lt)?)),
        }
    }

    fn check_lifetime(&mut self, lifetime: &Lifetime) -> Result<Lifetime> {
        match *lifetime {
            Lifetime::Var(depth) => {
                if depth >= self.binders {
                    // a free existentially bound region; find the
                    // inference variable it corresponds to
                    let v = LifetimeInferenceVariable::from_depth(depth - self.binders);
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
                            Ok(Lifetime::Var(depth))
                        }

                        InferenceValue::Bound(l) => {
                            Ok(l.up_shift(self.binders))
                        }
                    }
                } else {
                    // a bound region like `'a` in `for<'a> fn(&'a i32)`
                    Ok(Lifetime::Var(depth))
                }
            }
            Lifetime::ForAll(ui) => {
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
                    self.unifier.push_lifetime_eq_constraint(tick_x.to_lifetime(), *lifetime);
                    Ok(tick_x.to_lifetime())
                } else {
                    // If the `ui` is higher than `self.universe_index`, then we can name
                    // this lifetime, no problem.
                    Ok(Lifetime::ForAll(ui))
                }
            }
        }
    }

    fn check_ty(&mut self, parameter: &Ty) -> Result<Ty> {
        if let Some(n_parameter) = self.unifier.table.normalize_shallow(parameter) {
            return self.check_ty(&n_parameter);
        }

        match *parameter {
            Ty::Apply(ref parameter_apply) => {
                Ok(Ty::Apply(self.check_apply(parameter_apply)?))
            }

            Ty::ForAll(ref quantified_ty) => {
                Ok(Ty::ForAll(Box::new(self.check_quantified(quantified_ty)?)))
            }

            Ty::Var(depth) => {
                let v = TyInferenceVariable::from_depth(depth - self.binders);
                let ui = self.unifier.table.ty_unify.probe_value(v).unbound().unwrap();

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

                Ok(Ty::Var(depth))
            }

            Ty::Projection(ProjectionTy { associated_ty_id, ref parameters }) => {
                // FIXME(#6) -- this rejects constraints like
                // `exists(A -> A = Item0<<A as Item1>::foo>)`, which
                // is probably too conservative.
                let parameters =
                    parameters.iter()
                              .map(|p| self.check_parameter(p))
                              .collect::<Result<Vec<_>>>()?;
                Ok(Ty::Projection(ProjectionTy { associated_ty_id, parameters }))
            }
        }
    }

    fn universe_check(&mut self,
                      application_universe_index: UniverseIndex)
                      -> Result<()> {
        debug!("universe_check({:?}, {:?})",
               self.universe_index,
               application_universe_index);
        if self.universe_index < application_universe_index {
            bail!("incompatible universes(universe_index={:?}, application_universe_index={:?})",
                  self.universe_index,
                  application_universe_index)
        } else {
            Ok(())
        }
    }
}
