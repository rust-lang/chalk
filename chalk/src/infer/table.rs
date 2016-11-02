use ena::unify;
use super::leaf::*;
use super::universe::UniverseIndex;
use super::var::{InferenceVariable, InferenceValue};

pub struct InferenceTable {
    unify: unify::UnificationTable<InferenceVariable>,
    values: Vec<InferenceApplication>,
}

pub struct InferenceSnapshot {
    unify_snapshot: unify::Snapshot<InferenceVariable>,
    values_len: usize,
}

pub type UnifyResult<T> = Result<T, UnifyError>;

pub enum UnifyError {
    Cycle,
    IncompatibleConstants(InferenceConstant, InferenceConstant),
    IncompatibleArity(InferenceConstant, usize, usize), // indicates prog is not well-typed
    IncompatibleArgument(InferenceConstant, usize, Box<UnifyError>),
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

    fn normalize_shallow(&mut self, leaf: &InferenceLeaf) -> InferenceLeaf {
        match leaf.kind {
            InferenceLeafKind::Variable(var) => {
                match self.unify.probe_value(var) {
                    InferenceValue::Unbound(_) => leaf.clone(),
                    InferenceValue::Bound(val) => {
                        let application = self.values[val.as_usize()].clone();
                        InferenceLeaf::new(InferenceLeafData {
                            kind: InferenceLeafKind::Application(application),
                        })
                    }
                }
            }
            _ => leaf.clone(),
        }
    }

    pub fn unify(&mut self, leaf1: &InferenceLeaf, leaf2: &InferenceLeaf) -> UnifyResult<()> {
        self.commit_if_ok(|this| this.unify_in_snapshot(leaf1, leaf2))
    }

    fn unify_in_snapshot(&mut self,
                         leaf1: &InferenceLeaf,
                         leaf2: &InferenceLeaf)
                         -> UnifyResult<()> {
        // Remove any immediate inference variables.
        let leaf1 = self.normalize_shallow(leaf1);
        let leaf2 = self.normalize_shallow(leaf2);

        match (&leaf1.kind, &leaf2.kind) {
            (&InferenceLeafKind::Variable(var1), &InferenceLeafKind::Variable(var2)) => {
                Ok(self.unify
                    .unify_var_var(var1, var2)
                    .expect("unification of two unbound variables cannot fail"))
            }

            (&InferenceLeafKind::Application(ref application),
             &InferenceLeafKind::Variable(var)) |
            (&InferenceLeafKind::Variable(var),
             &InferenceLeafKind::Application(ref application)) => {
                self.unify_var_application(var, application)
            }

            (&InferenceLeafKind::Application(ref application1),
             &InferenceLeafKind::Application(ref application2)) => {
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
                             application: &InferenceApplication)
                             -> UnifyResult<()> {
        // Determine the universe index associated with this
        // variable. This is basically a count of the number of
        // `forall` binders that had been introduced at the point
        // this variable was created -- though it may change over time
        // as the variable is unified.
        let universe_index = match self.unify.probe_value(var) {
            InferenceValue::Unbound(ui) => ui,
            InferenceValue::Bound(_) => panic!("`unify_var_application` invoked on bound var"),
        };

        self.occurs_check(var, universe_index, application)?;

        Ok(())
    }

    fn occurs_check(&mut self,
                    var: InferenceVariable,
                    universe_index: UniverseIndex,
                    application: &InferenceApplication)
                    -> UnifyResult<()> {
        for arg in &application.args {
            let arg = self.normalize_shallow(arg);
            match arg.kind {
                InferenceLeafKind::Application(ref c) => {
                    self.universe_check(universe_index, c.constant.universe_index)?;
                    self.occurs_check(var, universe_index, c)?;
                }

                InferenceLeafKind::Variable(v) => {
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
