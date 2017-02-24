use ena::unify as ena;
use errors::*;
use ir::*;
use std::sync::Arc;

mod instantiate;
mod lifetime_var;
mod query;
mod unify;
mod var;
#[cfg(test)] mod test;

pub use self::unify::UnificationResult;
use self::var::{TyInferenceVariable, KrateInferenceVariable, InferenceValue};
use self::lifetime_var::LifetimeInferenceVariable;

#[derive(Clone)]
pub struct InferenceTable {
    ty_unify: ena::UnificationTable<TyInferenceVariable>,
    ty_values: Vec<Arc<Ty>>,

    krate_unify: ena::UnificationTable<KrateInferenceVariable>,
    krate_values: Vec<Krate>,

    /// Unlike normal variables, we don't unify lifetime variables.
    /// Instead, we just keep track of the universe in which they were
    /// created.
    lifetime_vars: Vec<UniverseIndex>,
}

pub struct InferenceSnapshot {
    ty_unify_snapshot: ena::Snapshot<TyInferenceVariable>,
    ty_values_len: usize,

    krate_unify_snapshot: ena::Snapshot<KrateInferenceVariable>,
}

pub type ParameterInferenceVariable = ParameterKind<TyInferenceVariable,
                                                    LifetimeInferenceVariable,
                                                    KrateInferenceVariable>;

impl InferenceTable {
    pub fn new() -> Self {
        InferenceTable {
            ty_unify: ena::UnificationTable::new(),
            krate_unify: ena::UnificationTable::new(),
            ty_values: vec![],
            krate_values: vec![],
            lifetime_vars: vec![],
        }
    }

    pub fn new_with_vars(vars: &[ParameterKind<UniverseIndex>]) -> Self {
        let mut table = InferenceTable::new();
        for &ui in vars {
            table.new_parameter_variable(ui);
        }
        table
    }

    pub fn new_variable(&mut self, ui: UniverseIndex) -> TyInferenceVariable {
        self.ty_unify.new_key(InferenceValue::Unbound(ui))
    }

    pub fn new_lifetime_variable(&mut self, ui: UniverseIndex) -> LifetimeInferenceVariable {
        let index = self.lifetime_vars.len();
        self.lifetime_vars.push(ui);
        LifetimeInferenceVariable::from_depth(index)
    }

    pub fn new_krate_variable(&mut self, ui: UniverseIndex) -> KrateInferenceVariable {
        self.krate_unify.new_key(InferenceValue::Unbound(ui))
    }

    pub fn new_parameter_variable(&mut self, ui: ParameterKind<UniverseIndex>)
                                  -> ParameterInferenceVariable {
        match ui {
            ParameterKind::Ty(ui) => ParameterKind::Ty(self.new_variable(ui)),
            ParameterKind::Lifetime(ui) => ParameterKind::Lifetime(self.new_lifetime_variable(ui)),
            ParameterKind::Krate(ui) => ParameterKind::Krate(self.new_krate_variable(ui)),
        }
    }

    fn lifetime_universe(&mut self, var: LifetimeInferenceVariable) -> UniverseIndex {
        self.lifetime_vars[var.to_usize()]
    }

    pub fn snapshot(&mut self) -> InferenceSnapshot {
        let ty_unify_snapshot = self.ty_unify.snapshot();
        let krate_unify_snapshot = self.krate_unify.snapshot();
        InferenceSnapshot {
            ty_unify_snapshot,
            krate_unify_snapshot,
            ty_values_len: self.ty_values.len(),
        }
    }

    pub fn rollback_to(&mut self, snapshot: InferenceSnapshot) {
        self.ty_unify.rollback_to(snapshot.ty_unify_snapshot);
        self.ty_values.truncate(snapshot.ty_values_len);
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

    fn normalize_shallow(&mut self, leaf: &Ty) -> Option<Arc<Ty>> {
        leaf.inference_var()
            .and_then(|var| {
                match self.ty_unify.probe_value(var) {
                    InferenceValue::Unbound(_) => None,
                    InferenceValue::Bound(val) => Some(self.ty_values[val.as_usize()].clone()),
                }
            })
    }

    fn normalize_krate(&mut self, leaf: &Krate) -> Option<Krate> {
        match *leaf {
            Krate::Var(v) => self.probe_krate_var(KrateInferenceVariable::from_depth(v)),
            Krate::Id(_) => None,
        }
    }

    fn probe_var(&mut self, var: TyInferenceVariable) -> Option<Arc<Ty>> {
        match self.ty_unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(val) => Some(self.ty_values[val.as_usize()].clone()),
        }
    }

    fn probe_krate_var(&mut self, var: KrateInferenceVariable) -> Option<Krate> {
        match self.krate_unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(val) => Some(self.krate_values[val.as_usize()]),
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
