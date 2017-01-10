use cast::Cast;
use errors::*;
use fold::{Fold, Folder};
use ir::*;
use solve::Successful;
use solve::environment::{Environment, InEnvironment};
use solve::infer::{InferenceTable, UnificationResult, ParameterInferenceVariable};
use solve::solver::Solver;
use std::collections::HashSet;
use std::fmt::Debug;
use std::mem;
use std::sync::Arc;
use zip::Zip;

pub struct Fulfill<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    obligations: Vec<InEnvironment<WhereClause>>,
    constraints: HashSet<Constraint>,
}

impl<'s> Fulfill<'s> {
    pub fn new(solver: &'s mut Solver, infer: InferenceTable) -> Self {
        Fulfill { solver, infer, obligations: vec![], constraints: HashSet::new() }
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
        where T: Zip + Debug
    {
        let UnificationResult { normalizations, constraints } = self.infer.unify(a, b)?;
        debug!("unify({:?}, {:?}) succeeded", a, b);
        debug!("unify: normalizations={:?}", normalizations);
        debug!("unify: constraints={:?}", constraints);
        self.constraints.extend(constraints);
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

    /// Return current list of pending obligations; used for unit testing primarily
    pub fn pending_obligations(&self) -> &[InEnvironment<WhereClause>] {
        &self.obligations
    }

    /// Create obligations for the given goal in the given
    /// environment. This may ultimately create any number of
    /// obligations.
    pub fn push_goal(&mut self, goal: &Goal, environment: &Arc<Environment>) {
        self.push_goal_bindings(goal, environment, &mut vec![])
    }

    fn push_goal_bindings(&mut self,
                          goal: &Goal,
                          environment: &Arc<Environment>,
                          bindings: &mut Vec<Binding>) {
        match *goal {
            Goal::Quantified(QuantifierKind::ForAll, ref parameter_kind, ref subgoal) => {
                let new_environment = environment.clone().new_universe();
                let parameter_universe = parameter_kind.map(|()| new_environment.universe);
                bindings.push(Binding::ForAll(parameter_universe));
                self.push_goal_bindings(subgoal, &new_environment, bindings);
                bindings.pop().unwrap();
            }
            Goal::Quantified(QuantifierKind::Exists, ref parameter_kind, ref subgoal) => {
                let parameter_universe = parameter_kind.map(|()| environment.universe);
                let var = self.new_parameter_variable(parameter_universe);
                bindings.push(Binding::Exists(var));
                self.push_goal_bindings(subgoal, environment, bindings);
                bindings.pop().unwrap();
            }
            Goal::Implies(ref wc, ref subgoal) => {
                let wc = Subst::apply(&bindings, wc);
                let new_environment = &environment.add_clauses(wc);
                self.push_goal_bindings(subgoal, new_environment, bindings);
            }
            Goal::And(ref subgoal1, ref subgoal2) => {
                self.push_goal_bindings(subgoal1, environment, bindings);
                self.push_goal_bindings(subgoal2, environment, bindings);
            }
            Goal::Leaf(ref wc) => {
                let wc = Subst::apply(&bindings, wc);
                self.obligations.push(InEnvironment::new(environment, wc));
            }
        }
    }

    /// As the final step in process a goal, we always have to deliver
    /// back a "refined goal" that shows what we learned. This refined
    /// goal combines any lifetime constraints with the final results
    /// of inference. It is produced by this method.
    pub fn refine_goal<G: Fold>(mut self, goal: G) -> Quantified<Constrained<G::Result>> {
        let mut constraints: Vec<_> = self.constraints.into_iter().collect();
        constraints.sort();
        debug!("refine_goal: constraints = {:?}", constraints);
        let constrained_goal = Constrained {
            value: goal,
            constraints: constraints,
        };
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
        let new_type_info = {
            solution.refined_goal.binders != quantified_wc.binders ||
            solution.refined_goal.value.value != quantified_wc.value
        };

        if new_type_info || !solution.refined_goal.value.constraints.is_empty() {
            let Constrained { constraints, value: refined_goal } =
                self.instantiate(solution.refined_goal.binders.iter().cloned(),
                                 &solution.refined_goal.value);

            self.constraints.extend(constraints);

            if new_type_info {
                self.unify(&wc.environment, wc, &refined_goal)?;
                *inference_progress = true;
            }
        }

        Ok(solution.successful)
    }
}

enum Binding {
    ForAll(ParameterKind<UniverseIndex>),
    Exists(ParameterInferenceVariable),
}

struct Subst<'b> {
    bindings: &'b [Binding],
}

impl<'b> Subst<'b> {
    fn apply<T: Fold>(bindings: &[Binding], value: &T) -> T::Result {
        value.fold_with(&mut Subst { bindings: bindings }).unwrap()
    }
}

impl<'b> Folder for Subst<'b> {
    fn fold_var(&mut self, depth: usize) -> Result<Ty> {
        match self.bindings[depth] {
            Binding::ForAll(u) => {
                Ok(Ty::Apply(ApplicationTy {
                    name: TypeName::ForAll(u.ty().unwrap()),
                    parameters: vec![],
                }))
            }
            Binding::Exists(v) => Ok(v.ty().unwrap().to_ty()),
        }
    }

    fn fold_lifetime_var(&mut self, depth: usize) -> Result<Lifetime> {
        match self.bindings[depth] {
            Binding::ForAll(u) => Ok(Lifetime::ForAll(u.lifetime().unwrap())),
            Binding::Exists(v) => Ok(v.lifetime().unwrap().to_lifetime()),
        }
    }
}
