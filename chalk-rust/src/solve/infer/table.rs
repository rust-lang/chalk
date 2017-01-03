use ena::unify;
use errors::*;
use ir;
use std::borrow::Cow;
use super::universe::UniverseIndex;
use super::var::*;

#[derive(Clone)]
pub struct InferenceTable {
    unify: unify::UnificationTable<InferenceVariable>,
    values: Vec<Arc<ir::Ty>>,
}

pub struct InferenceSnapshot {
    unify_snapshot: unify::Snapshot<InferenceVariable>,
    values_len: usize,
}

pub type UnifyResult<T> = Result<T, UnifyError>;

impl InferenceTable {
    pub fn new() -> Self {
        InferenceTable {
            unify: unify::UnificationTable::new(),
            values: vec![],
        }
    }

    pub fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        self.unify.new_key(InferenceValue::Unbound(ui))
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

    fn commit_if_ok<F, R>(&mut self, op: F) -> UnifyResult<R>
        where F: FnOnce(&mut Self) -> UnifyResult<R>
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

    fn normalize_shallow(&mut self, leaf: &ir::Ty) -> Option<Arc<ir::Ty>> {
        leaf.inference_var()
            .and_then(|var| {
                match self.unify.probe_value(var) {
                    InferenceValue::Unbound(_) => None,
                    InferenceValue::Bound(val) => Some(self.values[val.as_usize()].clone()),
                }
            })
    }

    fn probe_var(&mut self, var: InferenceVariable) -> Option<Arc<ir::Ty>> {
        match self.unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(val) => Some(self.values[val.as_usize()].clone()),
        }
    }
}

impl ir::Ty {
    pub fn inference_var(&self) -> Option<InferenceVariable> {
        if let ir::Ty::Var { depth } = *self {
            Some(InferenceVariable::from_depth(depth))
        } else {
            None
        }
    }

    pub fn universe_index(&self) -> Option<InferenceVariable> {
        match *self {
            ir::Ty::Var
        if let ir::Ty::Var { depth } = *self {
            Some(InferenceVariable::from_depth(depth))
        } else {
            None
        }
    }
}

struct Unifier<'t> {
    table: &'t mut InferenceTable,
    snapshot: InferenceSnapshot,
}

impl<'t> Unifier {
    pub fn new(table: &'t mut InferenceTable) {
        let snapshot = table.snapshot();
        Unifier {
            table: table,
            snapshot: snapshot,
        }
    }

    pub fn unify_ty_ty(&mut self, a: &ir::Ty, b: &ir::Ty) -> Result<()> {
        if let Some(n_a) = self.normalize_shallow(leaf1) {
            return self.unify_ty_ty(&n_a, b);
        } else if let Some(n_b) = self.normalize_shallow(leaf2) {
            return self.unify_ty_ty(a, &n_b);
        }

        debug!("unify_in_snapshot, normalized a={:?}", a);
        debug!("unify_in_snapshot, normalized b={:?}", b);

        match (a, b) {
            (&Ty::Var { depth: depth1 }, &Ty::Var { depth: depth2 }) => {
                let var1 = InferenceVariable::from_depth(depth1);
                let var2 = InferenceVariable::from_depth(depth2);
                debug!("unify_in_snapshot: unify_var_var({:?}, {:?})", var1, var2);
                Ok(self.unify
                   .unify_var_var(var1, var2)
                   .expect("unification of two unbound variables cannot fail"))
            }

            (&Ty::Var { depth }, ty) | (ty, &Ty::Var { depth }) =>
                self.unify_var_ty(InferenceVariable::from_depth(depth), ty),
        }
    }

    fn unify_var_ty(&mut self, var: InferenceVariable, ty: &ir::Ty) -> Result<()> {
        debug!("unify_var_ty(var={:?}, ty={:?})", var, ty);

        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = match self.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("`unify_var_application` invoked on bound var"),
        };

        self.universe_check(universe_index, ty.universe_index())?;
        self.occurs_check(var, universe_index, application)?;
    }
}

