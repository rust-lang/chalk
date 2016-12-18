use ena::unify;
use formula::*;
use super::universe::UniverseIndex;
use super::var::*;

#[derive(Clone)]
pub struct InferenceTable {
    unify: unify::UnificationTable<InferenceVariable>,
    values: Vec<Application>,
}

pub struct InferenceSnapshot {
    unify_snapshot: unify::Snapshot<InferenceVariable>,
    values_len: usize,
}

pub type UnifyResult<T> = Result<T, UnifyError>;

#[derive(Debug)]
pub enum UnifyError {
    Cycle,
    IncompatibleConstants(Constant, Constant),
    IncompatibleArity(Constant, usize, usize), // indicates prog is not well-typed
    IncompatibleArgument(Constant, usize, Box<UnifyError>),
    IncompatibleUniverses(UniverseIndex, UniverseIndex),
}

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

    fn normalize_shallow(&mut self, leaf: &Leaf) -> Leaf {
        match leaf.kind {
            LeafKind::InferenceVariable(var) => {
                match self.unify.probe_value(var) {
                    InferenceValue::Unbound(_) => leaf.clone(),
                    InferenceValue::Bound(val) => {
                        let application = self.values[val.as_usize()].clone();
                        Leaf::new(LeafData { kind: LeafKind::Application(application) })
                    }
                }
            }
            _ => leaf.clone(),
        }
    }

    pub fn normalize_deep<F: Fold>(&mut self, leaf: &F) -> F {
        leaf.fold_with(self)
    }

    pub fn unify(&mut self, leaf1: &Leaf, leaf2: &Leaf) -> UnifyResult<()> {
        self.commit_if_ok(|this| this.unify_in_snapshot(leaf1, leaf2))
    }

    fn unify_in_snapshot(&mut self, leaf1: &Leaf, leaf2: &Leaf) -> UnifyResult<()> {
        // Remove any immediate inference variables.
        let leaf1 = self.normalize_shallow(leaf1);
        let leaf2 = self.normalize_shallow(leaf2);

        debug!("unify_in_snapshot, normalized leaf1={:?}", leaf1);
        debug!("unify_in_snapshot, normalized leaf2={:?}", leaf2);

        match (&leaf1.kind, &leaf2.kind) {
            (&LeafKind::BoundVariable(_), _) |
            (_, &LeafKind::BoundVariable(_)) => {
                panic!("asked to unify bound variables: {:?} vs {:?}", leaf1, leaf2);
            }
            (&LeafKind::InferenceVariable(var1), &LeafKind::InferenceVariable(var2)) => {
                debug!("unify_in_snapshot: unify_var_var({:?}, {:?})", var1, var2);
                Ok(self.unify
                    .unify_var_var(var1, var2)
                    .expect("unification of two unbound variables cannot fail"))
            }

            (&LeafKind::Application(ref application), &LeafKind::InferenceVariable(var)) |
            (&LeafKind::InferenceVariable(var), &LeafKind::Application(ref application)) => {
                self.unify_var_application(var, application)
            }

            (&LeafKind::Application(ref application1),
             &LeafKind::Application(ref application2)) => {
                if application1.constant != application2.constant {
                    return Err(UnifyError::IncompatibleConstants(application1.constant,
                                                                 application2.constant));
                }

                if application1.args.len() != application2.args.len() {
                    return Err(UnifyError::IncompatibleArity(application1.constant,
                                                             application1.args.len(),
                                                             application2.args.len()));
                }

                let zipped_args = application1.args.iter().zip(&application2.args).enumerate();
                for (index, (arg1, arg2)) in zipped_args {
                    match self.unify(arg1, arg2) {
                        Ok(()) => (),
                        Err(err) => {
                            return Err(UnifyError::IncompatibleArgument(application1.constant,
                                                                        index,
                                                                        Box::new(err)))
                        }
                    }
                }

                Ok(())
            }
        }
    }

    /// Unify `var` with the application `application`. `var` must be in an
    /// unbound state already.
    fn unify_var_application(&mut self,
                             var: InferenceVariable,
                             application: &Application)
                             -> UnifyResult<()> {
        debug!("unify_var_application(var={:?}, application={:?})", var, application);

        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = match self.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("`unify_var_application` invoked on bound var"),
        };

        self.universe_check(universe_index, application.constant.universe_index())?;
        self.occurs_check(var, universe_index, application)?;

        let value_index = ValueIndex::new(self.values.len());
        self.values.push(application.clone());
        self.unify.unify_var_value(var, InferenceValue::Bound(value_index)).unwrap();

        debug!("unify_var_application: OK");

        Ok(())
    }

    fn occurs_check(&mut self,
                    var: InferenceVariable,
                    universe_index: UniverseIndex,
                    application: &Application)
                    -> UnifyResult<()> {
        for arg in &application.args {
            let arg = self.normalize_shallow(arg);
            match arg.kind {
                LeafKind::BoundVariable(_) => {
                    panic!("found bound variable in occurs check")
                }

                LeafKind::Application(ref c) => {
                    self.universe_check(universe_index, c.constant.universe_index())?;
                    self.occurs_check(var, universe_index, c)?;
                }

                LeafKind::InferenceVariable(v) => {
                    match self.unify.probe_value(v) {
                        InferenceValue::Unbound(ui) => {
                            self.universe_check(universe_index, ui)?;

                            if self.unify.unioned(v, var) {
                                return Err(UnifyError::Cycle);
                            }
                        }

                        InferenceValue::Bound(_) => {
                            unreachable!("expected `arg` to be normalized");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn universe_check(&mut self,
                      universe_index: UniverseIndex,
                      application_universe_index: UniverseIndex)
                      -> UnifyResult<()> {
        if universe_index < application_universe_index {
            Err(UnifyError::IncompatibleUniverses(universe_index, application_universe_index))
        } else {
            Ok(())
        }
    }
}

/// The inference table, when folding, normalizes bound variables with
/// their current values.
impl Folder for InferenceTable {
    fn in_binders<OP, R>(&mut self, _num_binders: usize, op: OP) -> R
        where OP: FnOnce(&mut Self) -> R
    {
        op(self)
    }

    fn replace_bound_variable(&mut self, from_leaf: &Leaf, _: BoundVariable) -> Leaf {
        from_leaf.clone()
    }

    fn replace_inference_variable(&mut self, from_leaf: &Leaf, var: InferenceVariable) -> Leaf {
        debug!("replace_inference_variable(from_leaf={:?}, var={:?})", from_leaf, var);
        let value = self.unify.probe_value(var);
        debug!("replace_inference_variable: value={:?}", value);
        match value {
            InferenceValue::Unbound(_) => {
                let root_var = self.unify.find(var);
                Leaf::new(LeafData { kind: LeafKind::InferenceVariable(root_var) })
            }
            InferenceValue::Bound(val) => {
                let application = self.values[val.as_usize()].clone();
                let leaf = Leaf::new(LeafData { kind: LeafKind::Application(application) });
                self.normalize_deep(&leaf)
            }
        }
    }
}
