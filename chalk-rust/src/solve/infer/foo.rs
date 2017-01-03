
    pub fn unify_var_var(&mut self, var1: InferenceVariable, var2: InferenceVariable) -> UnifyResult<()> {
        
    }

    fn unify_in_snapshot(&mut self, leaf1: &Leaf, leaf2: &Leaf) -> UnifyResult<()> {
        // Remove any immediate inference variables.
        if let Some(n_leaf1) = self.normalize_shallow(leaf1) {
            self.unify_in_snapshot(&n_leaf1, leaf2)
        } else if let Some(n_leaf2) = self.normalize_shallow(leaf2) {
            self.unify_in_snapshot(leaf1, &n_leaf2)
        } else {
            debug!("unify_in_snapshot, normalized leaf1={:?}", leaf1);
            debug!("unify_in_snapshot, normalized leaf2={:?}", leaf2);

            match (leaf1, leaf2) {
                (&Ty::Var { depth: depth1 }, &Ty::Var { depth: depth2 }) => {
                    let var1 = InferenceVariable::from_depth(depth1);
                    let var2 = InferenceVariable::from_depth(depth2);
                    debug!("unify_in_snapshot: unify_var_var({:?}, {:?})", var1, var2);
                    Ok(self.unify
                       .unify_var_var(var1, var2)
                       .expect("unification of two unbound variables cannot fail"))
                }

                (&Ty::Var { depth }, ty) | (ty, &Ty::Var { depth }) =>
                    self.unify_var_ty(var, ty),

                (&Ty::Apply { id: id1, args: ref args1 }, &Ty::Apply { id: id2, args: ref arg2 }) =>
                    self.unify_apply_apply(id1, args1, id2, args2),

                (&Ty::Projection { .. }, _) | (_, &Ty::Projection { .. }) =>
                    unimplemented!("projection"),
            }
        }
    }

    fn unify_apply_apply(&mut self,
                         id1: ir::ItemId,
                         args1: &[ir::Ty],
                         id2: ir::ItemId,
                         args2: &[ir::Ty])
                         -> Result<> {
        if id1 != id2 {
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

    /// Unify `var` with the application `application`. `var` must be in an
    /// unbound state already.
    fn unify_var_ty(&mut self,
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
                            if self.unify.unioned(v, var) {
                                return Err(UnifyError::Cycle);
                            }

                            if universe_index < ui {
                                // Scenario is like:
                                //
                                // ?A = foo(?B)
                                //
                                // where ?A is in universe 0 and ?B is in universe 1.
                                // This is OK, if ?B is promoted to universe 0.
                                self.unify.unify_var_value(v, InferenceValue::Unbound(universe_index)).unwrap();
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
        debug!("universe_check({:?}, {:?})", universe_index, application_universe_index);
        if universe_index < application_universe_index {
            bail!(ErrorKind::IncompatibleUniverses(universe_index, application_universe_index))
        } else {
            Ok(())
        }
    }
