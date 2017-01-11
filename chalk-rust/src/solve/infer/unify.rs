use cast::Cast;
use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
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
        where T: Zip
    {
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

        debug!("unify_ty_ty(normalized a={:?})", a);
        debug!("unify_ty_ty(normalized b={:?})", b);

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

            (&Ty::Var(depth), &Ty::Apply(ref apply)) |
            (&Ty::Apply(ref apply), &Ty::Var(depth)) => {
                self.unify_var_apply(InferenceVariable::from_depth(depth), apply)
            }

            (&Ty::Apply(ref apply1), &Ty::Apply(ref apply2)) => Zip::zip_with(self, apply1, apply2),

            (ty, &Ty::Projection(ref proj)) |
            (&Ty::Projection(ref proj), ty) => {
                Ok(self.goals.push(InEnvironment::new(self.environment, Normalize {
                    projection: proj.clone(),
                    ty: ty.clone(),
                }.cast())))
            }
        }
    }

    fn unify_var_apply(&mut self, var: InferenceVariable, apply: &ApplicationTy) -> Result<()> {
        debug!("unify_var_apply(var={:?}, apply={:?})", var, apply);

        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = match self.table.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("`unify_var_apply` invoked on bound var"),
        };

        self.universe_check(universe_index, apply.universe_index())?;
        self.occurs_check_apply(var, universe_index, apply)?;

        let value_index = ValueIndex::new(self.table.values.len());
        self.table.values.push(Arc::new(Ty::Apply(apply.clone())));
        self.table.unify.unify_var_value(var, InferenceValue::Bound(value_index)).unwrap();
        debug!("unify_var_apply: var {:?} set to {:?}", var, apply);

        Ok(())
    }

    fn universe_check(&mut self,
                      universe_index: UniverseIndex,
                      application_universe_index: UniverseIndex)
                      -> Result<()> {
        debug!("universe_check({:?}, {:?})",
               universe_index,
               application_universe_index);
        if universe_index < application_universe_index {
            bail!("incompatible universes(universe_index={:?}, application_universe_index={:?})",
                  universe_index,
                  application_universe_index)
        } else {
            Ok(())
        }
    }

    fn occurs_check_apply(&mut self,
                          var: InferenceVariable,
                          universe_index: UniverseIndex,
                          apply: &ApplicationTy)
                          -> Result<()> {
        for parameter in &apply.parameters {
            self.occurs_check_parameter(var, universe_index, parameter)?;
        }
        Ok(())
    }

    fn occurs_check_parameter(&mut self,
                              var: InferenceVariable,
                              universe_index: UniverseIndex,
                              arg: &Parameter)
                              -> Result<()> {
        match *arg {
            ParameterKind::Ty(ref t) => self.occurs_check_parameter_ty(var, universe_index, t),
            ParameterKind::Lifetime(_) => Ok(()),
        }
    }

    fn occurs_check_parameter_ty(&mut self,
                                 var: InferenceVariable,
                                 universe_index: UniverseIndex,
                                 parameter: &Ty)
                                 -> Result<()> {
        if let Some(n_parameter) = self.table.normalize_shallow(parameter) {
            return self.occurs_check_parameter_ty(var, universe_index, &n_parameter);
        }

        match *parameter {
            Ty::Apply(ref parameter_apply) => {
                self.universe_check(universe_index, parameter_apply.universe_index())?;
                self.occurs_check_apply(var, universe_index, parameter_apply)?;
            }

            Ty::Var(depth) => {
                let v = InferenceVariable::from_depth(depth);
                let ui = match self.table.unify.probe_value(v) {
                    InferenceValue::Unbound(ui) => ui,
                    InferenceValue::Bound(_) => {
                        unreachable!("expected `parameter` to be normalized")
                    }
                };

                if self.table.unify.unioned(v, var) {
                    bail!("cycle during unification");
                }

                if universe_index < ui {
                    // Scenario is like:
                    //
                    // ?A = foo(?B)
                    //
                    // where ?A is in universe 0 and ?B is in universe 1.
                    // This is OK, if ?B is promoted to universe 0.
                    self.table
                        .unify
                        .unify_var_value(v, InferenceValue::Unbound(universe_index))
                        .unwrap();
                }
            }

            Ty::Projection(ref proj) => {
                // FIXME(#6) -- this rejects constraints like
                // `exists(A -> A = Item0<<A as Item1>::foo>)`, which
                // is probably too conservative.
                for parameter in &proj.trait_ref.parameters {
                    self.occurs_check_parameter(var, universe_index, parameter)?;
                }
            }
        }
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
