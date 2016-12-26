use infer::*;
use formula::*;
use solve::*;
use subst::Subst;

pub type InstantiateResult<T> = Result<T, InstantiateError>;

pub enum InstantiateError {
    /// Not known to be a match, but maybe later.
    Ambiguous,

    /// Known not to be a match ever.
    Incompatible,
}


impl InferenceTable {
    pub fn instantiate_clause(&mut self,
                              environment: &Environment,
                              actual: &Application,
                              template: &Quantification<ClauseImplication<Application>>)
                              -> InstantiateResult<Option<Goal<Application>>> {
        debug!("instantiate_clause(actual={:?}, template={:?})",
               actual,
               template);

        let mut bindings: Vec<_> = (0..template.num_binders).map(|_| None).collect();
        let actual = self.normalize_deep(actual);

        // Figure out what we have to instantiate the bound variables
        // with in order to make `actual` equal to `template`. Note
        // that this never affects inference state, and may fail if
        // `actual` is insufficiently inferred.
        self.instantiate_appls(&mut bindings, &actual, &template.skip_binders().consequence)?;

        // Now that we know what to replace the bound variables with,
        // create a substitution with the values. If something was unconstrained,
        // create an inference variable.
        let mut subst = None;
        for b in bindings.into_iter().rev() {
            let v = b.unwrap_or_else(|| self.new_variable(environment.universe_index()).to_leaf());
            subst = Some(Subst::new(subst.as_ref(), v));
        }

        // Replace the bound variables in the consequence:
        let condition = &template.skip_binders().condition;
        Ok(subst.map(|s| s.apply(condition)).unwrap_or(condition.clone()))
    }

    fn instantiate_leaves(&mut self,
                          bindings: &mut Vec<Option<Leaf>>,
                          actual: &Leaf,
                          template: &Leaf)
                          -> InstantiateResult<()> {
        debug!("instantiate_leaves, actual={:?}", actual);
        debug!("instantiate_leaves, template={:?}", template);

        match (&actual.kind, &template.kind) {
            (&LeafKind::BoundVariable(_), _) => panic!("actual leaf contained bound variable"),
            (_, &LeafKind::BoundVariable(index)) => {
                let depth = index.depth();
                if bindings[depth].is_none() {
                    debug!("instantiate_appls: assigning {} to {:?}", depth, actual);
                    bindings[depth] = Some(actual.clone());
                    return Ok(());
                }

                let v = bindings[depth].as_ref().unwrap();
                debug!("instantiate_appls: already assigned {} to {:?} ({:?})",
                       depth,
                       v,
                       v == actual);
                if actual == v {
                    Ok(())
                } else {
                    // This is actually not true sometimes. e.g., if
                    // `v == Foo(?1)` and `actual == Bar(?1)`, then we
                    // could report `Incompatible`. But if `v ==
                    // Foo(?1)` and `actual == Foo(?2)`, then
                    // ambiguous would be the right answer.
                    Err(InstantiateError::Ambiguous)
                }
            }
            (_, &LeafKind::InferenceVariable(var2)) => {
                panic!("template leaf contained inference variable")
            }
            (&LeafKind::InferenceVariable(var1), _) => Err(InstantiateError::Ambiguous),
            (&LeafKind::Application(ref application1),
             &LeafKind::Application(ref application2)) => {
                self.instantiate_appls(bindings, application1, application2)
            }
        }
    }

    fn instantiate_appls(&mut self,
                         bindings: &mut Vec<Option<Leaf>>,
                         actual: &Application,
                         template: &Application)
                         -> InstantiateResult<()> {
        debug!("instantiate_appls(actual={:?}, template={:?})",
               actual,
               template);
        debug!("instantiate_appls: bindings={:?}", bindings);
        if actual.constant != template.constant || actual.args.len() != template.args.len() {
            return Err(InstantiateError::Incompatible);
        }

        let zipped_args = actual.args.iter().zip(&template.args);
        for (arg1, arg2) in zipped_args {
            self.instantiate_leaves(bindings, arg1, arg2)?;
        }

        Ok(())
    }
}
