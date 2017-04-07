use errors::*;
use fold::Fold;
use ir::*;
use solve::Successful;
use solve::infer::{InferenceTable, UnificationResult, ParameterInferenceVariable};
use solve::solver::Solver;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;
use zip::Zip;

pub struct Fulfill<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    obligations: Vec<InEnvironment<WhereClauseGoal>>,
    constraints: HashSet<InEnvironment<Constraint>>,
}

impl<'s> Fulfill<'s> {
    pub fn new(solver: &'s mut Solver, infer: InferenceTable) -> Self {
        Fulfill { solver, infer, obligations: vec![], constraints: HashSet::new() }
    }

    pub fn program(&self) -> Arc<ProgramEnvironment> {
        self.solver.program.clone()
    }

    /// Wraps `InferenceTable::instantiate`
    pub fn instantiate<U, T>(&mut self, universes: U, arg: &T) -> T::Result
        where T: Fold,
              U: IntoIterator<Item = ParameterKind<UniverseIndex>>
    {
        self.infer.instantiate(universes, arg)
    }

    /// Instantiates `arg` with fresh existential variables in the
    /// given universe; the kinds of the variables are implied by
    /// `binders`. This is used to apply a universally quantified
    /// clause like `forall X, 'Y. P => Q`. Here the `binders`
    /// argument is referring to `X, 'Y`.
    pub fn instantiate_in<U, T>(&mut self,
                                universe: UniverseIndex,
                                binders: U,
                                arg: &T) -> T::Result
        where T: Fold,
              U: IntoIterator<Item = ParameterKind<()>>
    {
        self.instantiate(binders.into_iter().map(|pk| pk.map(|_| universe)), arg)
    }

    /// Unifies `a` and `b` in the given environment.
    ///
    /// Wraps `InferenceTable::unify`; any resulting normalzations are
    /// added into our list of pending obligations with the given
    /// environment.
    pub fn unify<T>(&mut self, environment: &Arc<Environment>, a: &T, b: &T) -> Result<()>
        where T: ?Sized + Zip + Debug
    {
        let UnificationResult { goals, constraints } = self.infer.unify(environment, a, b)?;
        debug!("unify({:?}, {:?}) succeeded", a, b);
        debug!("unify: goals={:?}", goals);
        debug!("unify: constraints={:?}", constraints);
        self.constraints.extend(constraints);
        self.extend(goals);
        Ok(())
    }

    /// Wraps `InferenceTable::new_parameter_variable`
    pub fn new_parameter_variable(&mut self, ui: ParameterKind<UniverseIndex>)
                                  -> ParameterInferenceVariable {
        self.infer.new_parameter_variable(ui)
    }

    /// Adds the given where-clauses to the internal list of
    /// obligations that must be solved.
    pub fn extend<WC>(&mut self, wc: WC)
        where WC: IntoIterator<Item=InEnvironment<WhereClauseGoal>>
    {
        self.obligations.extend(wc);
    }

    /// Return current list of pending obligations; used for unit testing primarily
    pub fn pending_obligations(&self) -> &[InEnvironment<WhereClauseGoal>] {
        &self.obligations
    }

    /// Create obligations for the given goal in the given
    /// environment. This may ultimately create any number of
    /// obligations.
    pub fn push_goal(&mut self, goal: Goal, environment: &Arc<Environment>) {
        debug!("push_goal({:?}, {:?})", goal, environment);
        match goal {
            Goal::Quantified(QuantifierKind::ForAll, subgoal) => {
                let mut new_environment = environment.clone();
                let parameters: Vec<_> =
                    subgoal.binders
                           .iter()
                           .map(|pk| {
                               new_environment = new_environment.new_universe();
                               match *pk {
                                   ParameterKind::Lifetime(()) =>
                                       ParameterKind::Lifetime(Lifetime::ForAll(new_environment.universe)),

                                   ParameterKind::Ty(()) =>
                                       ParameterKind::Ty(Ty::Apply(ApplicationTy {
                                           name: TypeName::ForAll(new_environment.universe),
                                           parameters: vec![]
                                       })),

                                   ParameterKind::Krate(()) =>
                                       panic!("unimplemented: for-all binders with crates"),
                               }
                           })
                           .collect();
                let subgoal = subgoal.value.subst(&parameters);
                self.push_goal(subgoal, &new_environment);
            }
            Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                let subgoal = self.instantiate_in(environment.universe,
                                                  subgoal.binders.iter().cloned(),
                                                  &subgoal.value);
                self.push_goal(*subgoal, environment);
            }
            Goal::Implies(wc, subgoal) => {
                let new_environment = &environment.add_clauses(wc);
                self.push_goal(*subgoal, new_environment);
            }
            Goal::And(subgoal1, subgoal2) => {
                self.push_goal(*subgoal1, environment);
                self.push_goal(*subgoal2, environment);
            }
            Goal::Leaf(wc) => {
                self.obligations.push(InEnvironment::new(environment, wc));
            }
        }
    }

    /// As the final step in process a goal, we always have to deliver
    /// back a "refined goal" that shows what we learned. This refined
    /// goal combines any lifetime constraints with the final results
    /// of inference. It is produced by this method.
    pub fn refine_goal<G: Fold>(mut self, goal: G) -> Query<Constrained<G::Result>> {
        let mut constraints: Vec<_> = self.constraints.into_iter().collect();
        constraints.sort();
        debug!("refine_goal: constraints = {:?}", constraints);
        let constrained_goal = Constrained {
            value: goal,
            constraints: constraints,
        };
        self.infer.make_query(&constrained_goal)
    }

    /// Try to solve all of `where_clauses`, which may contain
    /// inference variables registered in the table `infer`. This can
    /// have side-effects on the inference state (regardless of
    /// whether it returns success, failure, or ambiguity). But, in
    /// all cases, the side-effects are only things that must be true
    /// for `where_clauses` to be true.
    pub fn solve_all(&mut self) -> Result<Successful> {
        debug_heading!("solve_all(where_clauses={:#?})", self.obligations);

        // Try to solve all the where-clauses. We do this via a
        // fixed-point iteration. We try to solve each where-clause in
        // turn. Anything which is successful, we drop; anything
        // ambiguous, we retain in the `where_clauses` array. This
        // process is repeated so long as we are learning new things
        // about our inference state.
        let mut obligations = Vec::with_capacity(self.obligations.len());
        let mut progress = true;
        while progress {
            progress = false;

            debug_heading!("start of round, {:?} obligations", self.obligations.len());

            // Take the list of `obligations` to solve this round and
            // replace it with an empty vector. Iterate through each
            // obligation to solve and solve it if we can. If not
            // (because of ambiguity), then push it back onto
            // `self.obligations` for next round. Note that
            // `solve_one` may also push onto the list.
            assert!(obligations.is_empty());
            while let Some(wc) = self.obligations.pop() {
                match self.solve_one(&wc, &mut progress)? {
                    Successful::Yes => (),
                    Successful::Maybe => {
                        debug!("ambiguous result: {:?}", wc);
                        obligations.push(wc);
                    }
                }
            }

            self.obligations.extend(obligations.drain(..));
            debug!("end of round, {:?} obligations left", self.obligations.len());
        }

        // At the end of this process, `self.obligations` should have
        // all of the ambiguous obligations, and `obligations` should
        // be empty.
        assert!(obligations.is_empty());

        // If we still have ambiguous where-clauses, then we have an
        // ambiguous overall result.
        if self.obligations.is_empty() {
            Ok(Successful::Yes)
        } else {
            debug!("still have {} ambiguous obligations: {:#?}",
                   self.obligations.len(), self.obligations);
            Ok(Successful::Maybe)
        }
    }

    fn solve_one(&mut self,
                 wc: &InEnvironment<WhereClauseGoal>,
                 inference_progress: &mut bool)
                 -> Result<Successful> {
        debug!("fulfill::solve_one(wc={:?})", wc);

        let quantified_wc = self.infer.make_query(&wc);
        let solution = self.solver.solve(quantified_wc.clone())?;

        // Regardless of whether the result is ambiguous or not,
        // solving the where-clause may have yielded a refined
        // goal. For example, if the original where-clause was
        // something like `Foo<?4>: Borrow<?3>`, we would have
        // "quantified" that to yield `exists ?0, ?1. Foo<?0>: Borrow<?1>`.
        // We may now have gotten back a refined goal like `exists ?0. Foo<?0>:
        // Borrow<Foo<?0>>`. In that case, we can unify `?3` with `Foo<?4>`.
        //
        // To make that unification happen, we first instantiate all
        // the variables on the goal we got back with new inference
        // variables. So we would thus convert `exists ?0. Foo<?0>:
        // Borrow<Foo<?0>>` into `Foo<?5>: Borrow<Foo<?5>>`.  We would
        // then unify this with our original goal (`Foo<?4>:
        // Borrow<?3>`). This will result in the equations `?4 = ?5`
        // and `?3 = Foo<?5>`.
        //
        // As a potential efficiency improvement, one could imagine a
        // more algorithm written just for this case instead of
        // instantiating with variables and applying the standard
        // unification algorithm. But this is good enough for now.
        let new_type_info = {
            solution.refined_goal.binders != quantified_wc.binders ||
            solution.refined_goal.value.value != quantified_wc.value
        };

        debug!("fulfill::solve_one: new_type_info={}", new_type_info);


        if new_type_info || !solution.refined_goal.value.constraints.is_empty() {
            let Constrained { constraints, value: refined_goal } =
                self.instantiate(universes_from_binders(&solution.refined_goal.binders),
                                 &solution.refined_goal.value);

            debug!("fulfill::solve_one: adding constraints {:?}", constraints);
            self.constraints.extend(constraints);

            debug!("fulfill::solve_one: unifying original and refined goal");
            self.unify(&wc.environment, wc, &refined_goal)?;

            if new_type_info {
                *inference_progress = true;
            }
        }

        Ok(solution.successful)
    }
}

fn universes_from_binders<'a>(binders: &'a QueryBinders) -> impl Iterator<Item = ParameterKind<UniverseIndex>> + 'a {
    let tys = binders.tys.iter().cloned().map(ParameterKind::Ty);
    let lifetimes = binders.lifetimes.iter().cloned().map(ParameterKind::Lifetime);
    let krates = binders.krates.iter().cloned().map(ParameterKind::Krate);
    tys.chain(lifetimes).chain(krates)
}
