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
    krate_unify: ena::UnificationTable<KrateInferenceVariable>,
    lifetime_unify: ena::UnificationTable<LifetimeInferenceVariable>,
}

pub struct InferenceSnapshot {
    ty_unify_snapshot: ena::Snapshot<TyInferenceVariable>,

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
            lifetime_unify: ena::UnificationTable::new(),
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
        self.lifetime_unify.new_key(InferenceValue::Unbound(ui))
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

    pub fn snapshot(&mut self) -> InferenceSnapshot {
        let ty_unify_snapshot = self.ty_unify.snapshot();
        let krate_unify_snapshot = self.krate_unify.snapshot();
        InferenceSnapshot {
            ty_unify_snapshot,
            krate_unify_snapshot,
        }
    }

    pub fn rollback_to(&mut self, snapshot: InferenceSnapshot) {
        self.ty_unify.rollback_to(snapshot.ty_unify_snapshot);
        self.krate_unify.rollback_to(snapshot.krate_unify_snapshot);
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

    fn normalize_krate(&mut self, leaf: &Krate) -> Option<Krate> {
        match *leaf {
            Krate::Var(v) => self.probe_krate_var(KrateInferenceVariable::from_depth(v)),
            Krate::Id(_) => None,
        }
    }

    fn probe_var(&mut self, var: TyInferenceVariable) -> Option<Ty> {
        match self.ty_unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(ref val) => Some(val.clone()),
        }
    }

    fn probe_krate_var(&mut self, var: KrateInferenceVariable) -> Option<Krate> {
        match self.krate_unify.probe_value(var) {
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
