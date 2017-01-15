use cast::Cast;
use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
use std::fmt::Debug;
use std::sync::Arc;
use zip::{Zip, Zipper};

use super::{InferenceSnapshot, InferenceTable, InferenceVariable};
use super::var::{InferenceValue, ValueIndex};

impl InferenceTable {
    pub fn unify<T>(&mut self,
                    environment: &Arc<Environment>,
                    a: &T,
                    b: &T)
                    -> Result<UnificationResult>
        where T: Zip + Debug,
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
    goals: Vec<InEnvironment<WhereClauseGoal>>,
    constraints: Vec<Constraint>,
}

#[derive(Debug)]
pub struct UnificationResult {
    pub goals: Vec<InEnvironment<WhereClauseGoal>>,
    pub constraints: Vec<Constraint>,
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
        }
    }

    fn commit(self) -> Result<UnificationResult> {
        self.table.commit(self.snapshot);
        Ok(UnificationResult {
            goals: self.goals,
            constraints: self.constraints,
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
                let var1 = InferenceVariable::from_depth(depth1);
                let var2 = InferenceVariable::from_depth(depth2);
                debug!("unify_ty_ty: unify_var_var({:?}, {:?})", var1, var2);
                Ok(self.table
                    .unify
                    .unify_var_var(var1, var2)
                    .expect("unification of two unbound variables cannot fail"))
            }

            (&Ty::Var(depth), ty @ &Ty::Apply(_)) |
            (ty @ &Ty::Apply(_), &Ty::Var(depth)) |
            (&Ty::Var(depth), ty @ &Ty::ForAll(_)) |
            (ty @ &Ty::ForAll(_), &Ty::Var(depth)) => {
                self.unify_var_ty(InferenceVariable::from_depth(depth), ty)
            }

            (&Ty::ForAll(ref quantified_ty1), &Ty::ForAll(ref quantified_ty2)) => {
                self.unify_forall_tys(quantified_ty1, quantified_ty2)
            }

            (&Ty::ForAll(ref quantified_ty), apply_ty @ &Ty::Apply(_)) |
            (apply_ty @ &Ty::Apply(_), &Ty::ForAll(ref quantified_ty)) => {
                self.unify_forall_apply(quantified_ty, apply_ty)
            }

            (&Ty::Apply(ref apply1), &Ty::Apply(ref apply2)) => Zip::zip_with(self, apply1, apply2),

            (ty, &Ty::Projection(ref proj)) |
            (&Ty::Projection(ref proj), ty) => {
                Ok(self.goals.push(InEnvironment::new(self.environment,
                                                      Normalize {
                                                              projection: proj.clone(),
                                                              ty: ty.clone(),
                                                          }
                                                          .cast())))
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

        let ty1 = ty1.instantiate(&lifetimes1);
        let ty2 = ty2.instantiate(&lifetimes2);
        debug!("unify_forall_tys: ty1 = {:?}", ty1);
        debug!("unify_forall_tys: ty2 = {:?}", ty2);

        let goal = InEnvironment::new(&environment, Unify { a: ty1, b: ty2 }).cast();
        debug!("unify_forall_tys: goal = {:?}", goal);

        self.goals.push(goal);

        Ok(())
    }

    fn unify_forall_apply(&mut self, ty1: &QuantifiedTy, ty2: &Ty) -> Result<()> {
        let mut environment = self.environment.clone();
        let lifetimes1: Vec<_> = (0..ty1.num_binders)
            .map(|_| {
                environment = environment.new_universe();
                Lifetime::ForAll(environment.universe).cast()
            })
            .collect();

        let ty1 = ty1.instantiate(&lifetimes1);
        let ty2 = ty2.clone();

        self.goals.push(InEnvironment::new(&environment, Unify { a: ty1, b: ty2 }).cast());

        Ok(())
    }

    fn unify_var_ty(&mut self, var: InferenceVariable, ty: &Ty) -> Result<()> {
        debug!("unify_var_ty(var={:?}, ty={:?})", var, ty);

        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = match self.table.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("`unify_var_apply` invoked on bound var"),
        };

        OccursCheck::new(self, var, universe_index).check_ty(ty)?;

        let value_index = ValueIndex::new(self.table.values.len());
        self.table.values.push(Arc::new(ty.clone()));
        self.table.unify.unify_var_value(var, InferenceValue::Bound(value_index)).unwrap();
        debug!("unify_var_ty: var {:?} set to {:?}", var, ty);

        Ok(())
    }
}

impl<'t> Zipper for Unifier<'t> {
    fn zip_tys(&mut self, a: &Ty, b: &Ty) -> Result<()> {
        self.unify_ty_ty(a, b)
    }

    fn zip_lifetimes(&mut self, &a: &Lifetime, &b: &Lifetime) -> Result<()> {
        Ok(self.constraints.push(Constraint::LifetimeEq(a, b)))
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

///////////////////////////////////////////////////////////////////////////

struct OccursCheck<'u, 't: 'u> {
    unifier: &'u mut Unifier<'t>,
    binders: usize,
    var: InferenceVariable,
    universe_index: UniverseIndex,
}

impl<'u, 't> OccursCheck<'u, 't> {
    fn new(unifier: &'u mut Unifier<'t>,
           var: InferenceVariable,
           universe_index: UniverseIndex)
           -> Self {
        OccursCheck { unifier, binders: 0, var, universe_index }
    }

    fn check_apply(&mut self, apply: &ApplicationTy) -> Result<()> {
        self.universe_check(apply.universe_index())?;
        for parameter in &apply.parameters {
            self.check_parameter(parameter)?;
        }
        Ok(())
    }

    fn check_quantified(&mut self, quantified_ty: &QuantifiedTy) -> Result<()> {
        self.binders += quantified_ty.num_binders;
        self.check_ty(&quantified_ty.ty)?;
        self.binders -= quantified_ty.num_binders;
        Ok(())
    }

    fn check_parameter(&mut self, arg: &Parameter) -> Result<()> {
        match *arg {
            ParameterKind::Ty(ref t) => self.check_ty(t),
            ParameterKind::Lifetime(_) => Ok(()),
        }
    }

    fn check_ty(&mut self, parameter: &Ty) -> Result<()> {
        if let Some(n_parameter) = self.unifier.table.normalize_shallow(parameter) {
            return self.check_ty(&n_parameter);
        }

        match *parameter {
            Ty::Apply(ref parameter_apply) => {
                self.check_apply(parameter_apply)?;
            }

            Ty::ForAll(ref quantified_ty) => {
                self.check_quantified(quantified_ty)?;
            }

            Ty::Var(depth) => {
                let v = InferenceVariable::from_depth(depth - self.binders);
                let ui = match self.unifier.table.unify.probe_value(v) {
                    InferenceValue::Unbound(ui) => ui,
                    InferenceValue::Bound(_) => {
                        unreachable!("expected `parameter` to be normalized")
                    }
                };

                if self.unifier.table.unify.unioned(v, self.var) {
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
                        .unify
                        .unify_var_value(v, InferenceValue::Unbound(self.universe_index))
                        .unwrap();
                }
            }

            Ty::Projection(ref proj) => {
                // FIXME(#6) -- this rejects constraints like
                // `exists(A -> A = Item0<<A as Item1>::foo>)`, which
                // is probably too conservative.
                for parameter in &proj.trait_ref.parameters {
                    self.check_parameter(parameter)?;
                }
            }
        }
        Ok(())
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
