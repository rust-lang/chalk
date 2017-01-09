use cast::Cast;
use errors::*;
use fold::Fold;
use ir::*;
use solve::Successful;
use solve::environment::{Environment, InEnvironment};
use solve::infer::{InferenceTable, UnificationResult, ParameterInferenceVariable};
use solve::solver::Solver;
use std::mem;
use std::sync::Arc;
use zip::Zip;

pub struct Fulfill<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    obligations: Vec<InEnvironment<WhereClause>>,
}

impl<'s> Fulfill<'s> {
    pub fn new(solver: &'s mut Solver, infer: InferenceTable) -> Self {
        let obligations = vec![];
        Fulfill { solver, infer, obligations }
    }

    pub fn program(&self) -> Arc<Program> {
        self.solver.program.clone()
    }

    /// Wraps `InferenceTable::instantiate`
    pub fn instantiate<U, T>(&mut self, universes: U, arg: &T) -> T::Result
        where T: Fold,
              U: IntoIterator<Item = ParameterKind<UniverseIndex>>
    {
        self.infer.instantiate(universes, arg)
    }

    /// Unifies `a` and `b` in the given environment.
    ///
    /// Wraps `InferenceTable::unify`; any resulting normalzations are
    /// added into our list of pending obligations with the given
    /// environment.
    pub fn unify<T>(&mut self, environment: &Arc<Environment>, a: &T, b: &T) -> Result<()>
        where T: Zip
    {
        let UnificationResult { normalizations } = self.infer.unify(a, b)?;
        self.extend(normalizations
                    .into_iter()
                    .map(|wc| InEnvironment::new(environment, wc.cast())));
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
        where WC: IntoIterator<Item=InEnvironment<WhereClause>>
    {
        self.obligations.extend(wc);
    }

    /// As the final step in process a goal, we always have to deliver
    /// back a "refined goal" that shows what we learned. This refined
    /// goal combines any lifetime constraints with the final results
    /// of inference. It is produced by this method.
    pub fn refine_goal<G: Fold>(mut self, goal: G) -> Quantified<Constrained<G::Result>> {
        let constrained_goal = self.infer.constrained(goal);
        self.infer.quantify(&constrained_goal)
    }

    /// Try to solve all of `where_clauses`, which may contain
    /// inference variables registered in the table `infer`. This can
    /// have side-effects on the inference state (regardless of
    /// whether it returns success, failure, or ambiguity). But, in
    /// all cases, the side-effects are only things that must be true
    /// for `where_clauses` to be true.
    pub fn solve_all(&mut self) -> Result<Successful> {
        debug!("solve_all(where_clauses={:#?})", self.obligations);

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

            // Take the list of `obligations` to solve this round and
            // replace it with an empty vector. Iterate through each
            // obligation to solve and solve it if we can. If not
            // (because of ambiguity), then push it back onto
            // `self.obligations` for next round. Note that
            // `solve_one` may also push onto the list.
            assert!(obligations.is_empty());
            mem::swap(&mut obligations, &mut self.obligations);
            for wc in obligations.drain(..) {
                match self.solve_one(&wc, &mut progress)? {
                    Successful::Yes => (),
                    Successful::Maybe => self.obligations.push(wc),
                }
            }
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
            Ok(Successful::Maybe)
        }
    }

    fn solve_one(&mut self,
                 wc: &InEnvironment<WhereClause>,
                 inference_progress: &mut bool)
                 -> Result<Successful> {
        let quantified_wc = self.infer.quantify(&wc);
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
        if {
            solution.refined_goal.binders != quantified_wc.binders ||
            solution.refined_goal.value.value != quantified_wc.value
        } {
            let refined_goal =
                self.instantiate(quantified_wc.binders.iter().cloned(),
                                 &solution.refined_goal.value);
            self.infer.unify(wc, &refined_goal.value)?; // FIXME
            *inference_progress = true;
        }

        Ok(solution.successful)
    }
}
