use ena::unify as ena;
use ir::*;
use ir::fold::Fold;
use ir::fold::shift::Shift;

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
crate struct InferenceTable {
    unify: ena::UnificationTable<InferenceVariable>,
    vars: Vec<InferenceVariable>,
    max_universe: UniverseIndex,
}

crate struct InferenceSnapshot {
    unify_snapshot: ena::Snapshot<InferenceVariable>,
    max_universe: UniverseIndex,
    vars: Vec<InferenceVariable>,
}

pub(in solve) type ParameterInferenceVariable = ParameterKind<InferenceVariable>;

impl InferenceTable {
    /// Create an empty inference table with no variables.
    crate fn new() -> Self {
        InferenceTable {
            unify: ena::UnificationTable::new(),
            vars: vec![],
            max_universe: UniverseIndex::root(),
        }
    }

    /// Creates a new inference table, pre-populated with
    /// `num_universes` fresh universes. Instantiates the canonical
    /// value `canonical` within those universes (which must not
    /// reference any universe greater than `num_universes`). Returns
    /// the substitution mapping from each canonical binder to its
    /// corresponding existential variable, along with the
    /// instantiated result.
    crate fn from_canonical<T>(
        num_universes: usize,
        canonical: &Canonical<T>,
    ) -> (Self, Substitution, T)
    where
        T: Fold<Result = T> + Clone,
    {
        let mut table = InferenceTable::new();

        assert!(num_universes >= 1); // always have U0
        for _ in 1..num_universes {
            table.new_universe();
        }

        let subst = table.fresh_subst(&canonical.binders);

        // Pointless micro-optimization: The fully correct way to
        // instantiate `value` is to substitute `subst` like so:
        //
        //     let value = canonical.substitute(&subst);
        //
        // However, because (a) this is a canonical value, and hence
        // contains no free variables except for those bound in the
        // canonical binders and (b) we just create the inference
        // table, and we created all of its variables from those same
        // binders, we know that this substitution will have the form
        // `?0 := ?0` and so forth.  So we can just "clone" the
        // canonical value rather than actually substituting.
        assert!(subst.is_identity_subst());
        let value = canonical.value.clone();

        (table, subst, value)
    }

    /// Creates and returns a fresh universe that is distinct from all
    /// others created within this inference table. This universe is
    /// able to see all previously created universes (though hopefully
    /// it is only brought into contact with its logical *parents*).
    crate fn new_universe(&mut self) -> UniverseIndex {
        let u = self.max_universe.next();
        self.max_universe = u;
        debug!("new_universe: {:?}", u);
        u
    }

    /// Current maximum universe -- one that can see all existing names.
    crate fn max_universe(&self) -> UniverseIndex {
        self.max_universe
    }

    /// Creates a new inference variable and returns its index. The
    /// kind of the variable should be known by the caller, but is not
    /// tracked directly by the inference table.
    pub(in solve) fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
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
                        Some(ty.shifted_in(binders))
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
                if v < binders {
                    return None;
                }
                let v1 = self.probe_lifetime_var(InferenceVariable::from_depth(v - binders))?;
                Some(v1.shifted_in(binders))
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
}

impl ParameterInferenceVariable {
    crate fn to_parameter(self) -> Parameter {
        match self {
            ParameterKind::Ty(v) => ParameterKind::Ty(v.to_ty()),
            ParameterKind::Lifetime(v) => ParameterKind::Lifetime(v.to_lifetime()),
        }
    }
}
