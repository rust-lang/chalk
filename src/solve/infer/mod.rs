use ena::unify as ena;
use errors::*;
use ir::*;

mod instantiate;
mod query;
mod unify;
mod var;
#[cfg(test)] mod test;

pub use self::unify::UnificationResult;
use self::var::*;

#[derive(Clone)]
pub struct InferenceTable {
    ty_unify: ena::UnificationTable<TyInferenceVariable>,
    lifetime_unify: ena::UnificationTable<LifetimeInferenceVariable>,
}

pub struct InferenceSnapshot {
    ty_unify_snapshot: ena::Snapshot<TyInferenceVariable>,
}

pub type ParameterInferenceVariable = ParameterKind<TyInferenceVariable, LifetimeInferenceVariable>;

impl InferenceTable {
    pub fn new() -> Self {
        InferenceTable {
            ty_unify: ena::UnificationTable::new(),
            lifetime_unify: ena::UnificationTable::new(),
        }
    }

    pub fn new_variable(&mut self, ui: UniverseIndex) -> TyInferenceVariable {
        self.ty_unify.new_key(InferenceValue::Unbound(ui))
    }

    pub fn new_lifetime_variable(&mut self, ui: UniverseIndex) -> LifetimeInferenceVariable {
        self.lifetime_unify.new_key(InferenceValue::Unbound(ui))
    }

    pub fn new_parameter_variable(&mut self, ui: ParameterKind<UniverseIndex>)
                                  -> ParameterInferenceVariable {
        match ui {
            ParameterKind::Ty(ui) => ParameterKind::Ty(self.new_variable(ui)),
            ParameterKind::Lifetime(ui) => ParameterKind::Lifetime(self.new_lifetime_variable(ui)),
        }
    }

    pub fn snapshot(&mut self) -> InferenceSnapshot {
        let ty_unify_snapshot = self.ty_unify.snapshot();
        InferenceSnapshot { ty_unify_snapshot }
    }

    pub fn rollback_to(&mut self, snapshot: InferenceSnapshot) {
        self.ty_unify.rollback_to(snapshot.ty_unify_snapshot);
    }

    fn commit(&mut self, snapshot: InferenceSnapshot) {
        self.ty_unify.commit(snapshot.ty_unify_snapshot);
    }

    fn commit_if_ok<F, R>(&mut self, op: F) -> Result<R>
        where F: FnOnce(&mut Self) -> Result<R>
    {
        let snapshot = self.snapshot();
        match op(self) {
            Ok(v) => {
                self.commit(snapshot);
                Ok(v)
            }

            Err(err) => {
                self.rollback_to(snapshot);
                Err(err)
            }
        }
    }

    fn normalize_shallow(&mut self, leaf: &Ty) -> Option<Ty> {
        leaf.inference_var()
            .and_then(|var| {
                match self.ty_unify.probe_value(var) {
                    InferenceValue::Unbound(_) => None,
                    InferenceValue::Bound(ref val) => Some(val.clone()),
                }
            })
    }

    fn normalize_lifetime(&mut self, leaf: &Lifetime) -> Option<Lifetime> {
        match *leaf {
            Lifetime::Var(v) => self.probe_lifetime_var(LifetimeInferenceVariable::from_depth(v)),
            Lifetime::ForAll(_) => None,
        }
    }

    fn probe_var(&mut self, var: TyInferenceVariable) -> Option<Ty> {
        match self.ty_unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(ref val) => Some(val.clone()),
        }
    }

    fn probe_lifetime_var(&mut self, var: LifetimeInferenceVariable) -> Option<Lifetime> {
        match self.lifetime_unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(val) => Some(val.clone()),
        }
    }
}

impl Ty {
    pub fn inference_var(&self) -> Option<TyInferenceVariable> {
        if let Ty::Var(depth) = *self {
            Some(TyInferenceVariable::from_depth(depth))
        } else {
            None
        }
    }
}
