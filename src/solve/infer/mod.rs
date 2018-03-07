use ena::unify as ena;
use ir::*;
use fold::shift::Shift;

crate mod canonicalize;
crate mod ucanonicalize;
mod normalize_deep;
crate mod instantiate;
mod invert;
crate mod unify;
crate mod var;
#[cfg(test)]
mod test;

use self::var::*;

#[derive(Clone)]
pub struct InferenceTable { // FIXME pub b/c of trait impl for SLG
    unify: ena::UnificationTable<InferenceVariable>,
    vars: Vec<InferenceVariable>,
    max_universe: UniverseIndex,
}

crate struct InferenceSnapshot {
    unify_snapshot: ena::Snapshot<InferenceVariable>,
    max_universe: UniverseIndex,
    vars: Vec<InferenceVariable>,
}

crate type ParameterInferenceVariable = ParameterKind<InferenceVariable>;

impl InferenceTable {
    /// Create an empty inference table with no variables.
    crate fn new() -> Self {
        InferenceTable {
            unify: ena::UnificationTable::new(),
            vars: vec![],
            max_universe: UniverseIndex::root(),
        }
    }

    /// Creates and returns a fresh universe that is distinct from all
    /// others created within this inference table. This universe is
    /// able to see all previously created universes (though hopefully
    /// it is only brought into contact with its logical *parents*).
    crate fn new_universe(&mut self) -> UniverseIndex {
        let u = self.max_universe.next();
        self.max_universe = u;
        u
    }

    /// Creates and returns a fresh universe that is distinct from all
    /// others created within this inference table. This universe is
    /// able to see all previously created universes (though hopefully
    /// it is only brought into contact with its logical *parents*).
    crate fn instantiate_universes<'v, T>(&mut self, value: &'v UCanonical<T>) -> &'v Canonical<T> {
        let UCanonical { universes, canonical } = value;
        assert!(*universes >= 1); // always have U0
        for _ in 1 .. *universes {
            self.new_universe();
        }
        canonical
    }

    /// Current maximum universe -- one that can see all existing names.
    crate fn max_universe(&self) -> UniverseIndex {
        self.max_universe
    }

    /// Creates a new inference variable and returns its index. The
    /// kind of the variable should be known by the caller, but is not
    /// tracked directly by the inference table.
    crate fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        let var = self.unify.new_key(InferenceValue::Unbound(ui));
        self.vars.push(var);
        debug!("new_variable: var={:?} ui={:?}", var, ui);
        var
    }

    /// Takes a "snapshot" of the current state of the inference
    /// table.  Later, you must invoke either `rollback_to` or
    /// `commit` with that snapshot.  Snapshots can be nested, but you
    /// must respect a stack discipline (i.e., rollback or commit
    /// snapshots in reverse order of that with which they were
    /// created).
    crate fn snapshot(&mut self) -> InferenceSnapshot {
        let unify_snapshot = self.unify.snapshot();
        let vars = self.vars.clone();
        let max_universe = self.max_universe;
        InferenceSnapshot {
            unify_snapshot,
            max_universe,
            vars,
        }
    }

    /// Restore the table to the state it had when the snapshot was taken.
    crate fn rollback_to(&mut self, snapshot: InferenceSnapshot) {
        self.unify.rollback_to(snapshot.unify_snapshot);
        self.vars = snapshot.vars;
        self.max_universe = snapshot.max_universe;
    }

    /// Make permanent the changes made since the snapshot was taken.
    crate fn commit(&mut self, snapshot: InferenceSnapshot) {
        self.unify.commit(snapshot.unify_snapshot);
    }

    /// If type `leaf` is a free inference variable, and that variable has been
    /// bound, returns `Some(T)` where `T` is the type to which it has been bound.
    ///
    /// `binders` is the number of binders under which `leaf` appears;
    /// the return value will also be shifted accordingly so that it
    /// can appear under that same number of binders.
    crate fn normalize_shallow(&mut self, leaf: &Ty, binders: usize) -> Option<Ty> {
        leaf.var().and_then(|depth| {
            if depth < binders {
                None // bound variable, not an inference var
            } else {
                let var = InferenceVariable::from_depth(depth - binders);
                match self.unify.probe_value(var) {
                    InferenceValue::Unbound(_) => None,
                    InferenceValue::Bound(ref val) => {
                        let ty = val.as_ref().ty().unwrap();
                        Some(ty.up_shift(binders))
                    }
                }
            }
        })
    }

    /// If `leaf` represents an inference variable `X`, and `X` is bound,
    /// returns `Some(v)` where `v` is the value to which `X` is bound.
    crate fn normalize_lifetime(&mut self, leaf: &Lifetime, binders: usize) -> Option<Lifetime> {
        match *leaf {
            Lifetime::Var(v) => {
                let v1 = self.probe_lifetime_var(InferenceVariable::from_depth(v))?;
                Some(v1.up_shift(binders))
            }
            Lifetime::ForAll(_) => None,
        }
    }

    /// Finds the type to which `var` is bound, returning `None` if it is not yet
    /// bound.
    ///
    /// # Panics
    ///
    /// This method is only valid for inference variables of kind
    /// type. If this variable is of a different kind, then the
    /// function may panic.
    fn probe_ty_var(&mut self, var: InferenceVariable) -> Option<Ty> {
        match self.unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(ref val) => Some(val.as_ref().ty().unwrap().clone()),
        }
    }

    /// Finds the lifetime to which `var` is bound, returning `None` if it is not yet
    /// bound.
    ///
    /// # Panics
    ///
    /// This method is only valid for inference variables of kind
    /// lifetime. If this variable is of a different kind, then the function may panic.
    fn probe_lifetime_var(&mut self, var: InferenceVariable) -> Option<Lifetime> {
        match self.unify.probe_value(var) {
            InferenceValue::Unbound(_) => None,
            InferenceValue::Bound(ref val) => Some(val.as_ref().lifetime().unwrap().clone()),
        }
    }

    /// True if the given inference variable is bound to a value.
    crate fn var_is_bound(&mut self, var: InferenceVariable) -> bool {
        match self.unify.probe_value(var) {
            InferenceValue::Unbound(_) => false,
            InferenceValue::Bound(_) => true,
        }
    }

    /// Given an unbound variable, returns its universe.
    ///
    /// # Panics
    ///
    /// Panics if the variable is bound.
    fn universe_of_unbound_var(&mut self, var: InferenceVariable) -> UniverseIndex {
        match self.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("var_universe invoked on bound variable"),
        }
    }
}

impl Ty {
    /// If this is a `Ty::Var(d)`, returns `Some(d)` else `None`.
    crate fn var(&self) -> Option<usize> {
        if let Ty::Var(depth) = *self {
            Some(depth)
        } else {
            None
        }
    }

    /// If this is a `Ty::Var`, returns the
    /// `TyInferenceVariable` it represents. Only makes sense if
    /// `self` is known not to appear inside of any binders, since
    /// otherwise the depth would have be adjusted to account for
    /// those binders.
    crate fn inference_var(&self) -> Option<InferenceVariable> {
        self.var().map(InferenceVariable::from_depth)
    }
}

impl Lifetime {
    /// If this is a `Lifetime::Var(d)`, returns `Some(d)` else `None`.
    crate fn var(&self) -> Option<usize> {
        if let Lifetime::Var(depth) = *self {
            Some(depth)
        } else {
            None
        }
    }

    /// If this is a `Lifetime::Var`, returns the
    /// `LifetimeInferenceVariable` it represents. Only makes sense if
    /// `self` is known not to appear inside of any binders, since
    /// otherwise the depth would have be adjusted to account for
    /// those binders.
    crate fn inference_var(&self) -> Option<InferenceVariable> {
        self.var().map(InferenceVariable::from_depth)
    }
}

impl Substitution {
    /// Check whether this substitution is the identity substitution in the
    /// given inference context.
    crate fn is_trivial_within(&self, in_infer: &mut InferenceTable) -> bool {
        for value in &self.parameters {
            match value {
                ParameterKind::Ty(ty) => if let Some(var) = ty.inference_var() {
                    if in_infer.var_is_bound(var) {
                        return false;
                    }
                },

                ParameterKind::Lifetime(lifetime) => if let Some(var) = lifetime.inference_var() {
                    if in_infer.var_is_bound(var) {
                        return false;
                    }
                },
            }
        }

        true
    }
}

impl ParameterInferenceVariable {
    crate fn to_parameter(self) -> Parameter {
        match self {
            ParameterKind::Ty(v) => ParameterKind::Ty(v.to_ty()),
            ParameterKind::Lifetime(v) => ParameterKind::Lifetime(v.to_lifetime()),
        }
    }
}
