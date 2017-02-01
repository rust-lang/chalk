use ena::unify as ena;
use errors::*;
use ir::*;
use std::sync::Arc;

mod instantiate;
mod lifetime_var;
mod quantify;
mod unify;
mod var;
#[cfg(test)] mod test;

pub use self::unify::UnificationResult;
use self::var::InferenceVariable;
use self::lifetime_var::LifetimeInferenceVariable;

use self::var::InferenceValue;

#[derive(Clone)]
pub struct InferenceTable {
    unify: ena::UnificationTable<InferenceVariable>,
    values: Vec<Arc<Ty>>,

    /// Unlike normal variables, we don't unify lifetime variables.
    /// Instead, we just keep track of the universe in which they were
    /// created.
    lifetime_vars: Vec<UniverseIndex>,
}

pub struct InferenceSnapshot {
    unify_snapshot: ena::Snapshot<InferenceVariable>,
    values_len: usize,
}

pub type ParameterInferenceVariable = ParameterKind<InferenceVariable, LifetimeInferenceVariable>;

impl InferenceTable {
    pub fn new() -> Self {
        InferenceTable {
            unify: ena::UnificationTable::new(),
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

    pub fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        self.unify.new_key(InferenceValue::Unbound(ui))
    }

    pub fn new_lifetime_variable(&mut self, ui: UniverseIndex) -> LifetimeInferenceVariable {
        let index = self.lifetime_vars.len();
        self.lifetime_vars.push(ui);
        LifetimeInferenceVariable::from_depth(index)
    }

    pub fn new_parameter_variable(&mut self, ui: ParameterKind<UniverseIndex>)
                                  -> ParameterInferenceVariable {
        match ui {
            ParameterKind::Ty(ui) => ParameterKind::Ty(self.new_variable(ui)),
            ParameterKind::Lifetime(ui) => ParameterKind::Lifetime(self.new_lifetime_variable(ui)),
        }
    }

    pub fn lifetime_universe(&mut self, var: LifetimeInferenceVariable) -> UniverseIndex {
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
                    InferenceValue::Unbound(_) => None,
                    InferenceValue::Bound(val) => Some(self.values[val.as_usize()].clone()),
                }
            })
    }

    fn probe_var(&mut self, var: InferenceVariable) -> Option<Arc<Ty>> {
        match self.unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(val) => Some(self.values[val.as_usize()].clone()),
        }
    }
}

impl Ty {
    pub fn inference_var(&self) -> Option<InferenceVariable> {
        if let Ty::Var(depth) = *self {
            Some(InferenceVariable::from_depth(depth))
        } else {
            None
        }
    }
}
