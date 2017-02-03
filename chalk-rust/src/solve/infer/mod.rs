use ena::unify as ena;
use errors::*;
use ir::*;
use std::sync::Arc;

mod krate_var;
mod instantiate;
mod lifetime_var;
mod quantify;
mod unify;
mod ty_var;
#[cfg(test)] mod test;

pub use self::unify::UnificationResult;
use self::ty_var::{TyInferenceVariable, TyInferenceValue};
use self::krate_var::{KrateInferenceVariable, KrateInferenceValue};
use self::lifetime_var::LifetimeInferenceVariable;

#[derive(Clone)]
pub struct InferenceTable {
    unify: ena::UnificationTable<TyInferenceVariable>,
    values: Vec<Arc<Ty>>,

    krate_unify: ena::UnificationTable<KrateInferenceVariable>,

    /// Unlike normal variables, we don't unify lifetime variables.
    /// Instead, we just keep track of the universe in which they were
    /// created.
    lifetime_vars: Vec<UniverseIndex>,
}

pub struct InferenceSnapshot {
    unify_snapshot: ena::Snapshot<TyInferenceVariable>,
    values_len: usize,
}

pub type ParameterInferenceVariable = ParameterKind<TyInferenceVariable,
                                                    LifetimeInferenceVariable,
                                                    KrateInferenceVariable>;

impl InferenceTable {
    pub fn new() -> Self {
        InferenceTable {
            unify: ena::UnificationTable::new(),
            krate_unify: ena::UnificationTable::new(),
            values: vec![],
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
        self.unify.new_key(TyInferenceValue::Unbound(ui))
    }

    pub fn new_lifetime_variable(&mut self, ui: UniverseIndex) -> LifetimeInferenceVariable {
        let index = self.lifetime_vars.len();
        self.lifetime_vars.push(ui);
        LifetimeInferenceVariable::from_depth(index)
    }

    pub fn new_krate_variable(&mut self, ui: UniverseIndex) -> KrateInferenceVariable {
        self.krate_unify.new_key(KrateInferenceValue::Unbound(ui))
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
        let unify_snapshot = self.unify.snapshot();
        InferenceSnapshot {
            unify_snapshot: unify_snapshot,
            values_len: self.values.len(),
        }
    }

    pub fn rollback_to(&mut self, snapshot: InferenceSnapshot) {
        self.unify.rollback_to(snapshot.unify_snapshot);
        self.values.truncate(snapshot.values_len);
    }

    fn commit(&mut self, snapshot: InferenceSnapshot) {
        self.unify.commit(snapshot.unify_snapshot);
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
                match self.unify.probe_value(var) {
                    TyInferenceValue::Unbound(_) => None,
                    TyInferenceValue::Bound(val) => Some(self.values[val.as_usize()].clone()),
                }
            })
    }

    fn probe_var(&mut self, var: TyInferenceVariable) -> Option<Arc<Ty>> {
        match self.unify.probe_value(var) {
            TyInferenceValue::Unbound(_) => None,
            TyInferenceValue::Bound(val) => Some(self.values[val.as_usize()].clone()),
        }
    }

    fn probe_krate_var(&mut self, var: KrateInferenceVariable) -> Option<Krate> {
        match self.krate_unify.probe_value(var) {
            KrateInferenceValue::Unbound(_) => None,
            KrateInferenceValue::Bound(id) => Some(Krate::Id(id)),
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
