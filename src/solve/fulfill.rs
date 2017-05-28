use super::*;
use errors::*;
use fold::Fold;
use solve::infer::{InferenceTable, UnificationResult, ParameterInferenceVariable};
use solve::infer::{TyInferenceVariable, LifetimeInferenceVariable};
use solve::solver::Solver;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;
use zip::Zip;

enum Outcome {
    Complete,
    Incomplete,
}

impl Outcome {
    fn is_complete(&self) -> bool {
        match *self {
            Outcome::Complete => true,
            _ => false,
        }
    }
}

/// A Fulfill is where we actually break down complex goals, instantiate
/// variables, and perform inference. It's highly stateful. It's generally used
/// in Chalk to try to solve a goal, and then package up what was learned in a
/// stateless, canonical way.
///
/// In rustc, you can think of there being an outermost Fulfill that's used when
/// type checking each function body, etc. There, the state reflects the state
/// of type inference in general. But when solving trait constraints, *fresh*
/// Fulfill instances will be created to solve canonicalized, free-standing
/// goals, and transport what was learned back to the outer context.
pub struct Fulfill<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,

    /// A flattened list of leaf goals to prove, each one in the given environment
    to_prove: Vec<InEnvironment<LeafGoal>>,

    /// Goals to refute; these aren't flattened because we want to isolate each
    /// set of subgoals into a fresh Fulfill
    to_refute: Vec<InEnvironment<Goal>>,

    /// Lifetime constraints that must be fulfilled for a solution to be fully
    /// validated.
    constraints: HashSet<InEnvironment<Constraint>>,
}

impl<'s> Fulfill<'s> {
    pub fn new(solver: &'s mut Solver) -> Self {
        Fulfill {
            solver,
            infer: InferenceTable::new(),
            to_prove: vec![],
            to_refute: vec![],
            constraints: HashSet::new()
        }
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
    /// Wraps `InferenceTable::unify`; any resulting normalzations are added
    /// into our list of pending obligations with the given environment.
    pub fn unify<T>(&mut self, environment: &Arc<Environment>, a: &T, b: &T) -> Result<()>
        where T: ?Sized + Zip + Debug
    {
        let UnificationResult { goals, constraints } = self.infer.unify(environment, a, b)?;
        debug!("unify({:?}, {:?}) succeeded", a, b);
        debug!("unify: goals={:?}", goals);
        debug!("unify: constraints={:?}", constraints);
        self.constraints.extend(constraints);
        self.to_prove.extend(goals);
        Ok(())
    }

    /// Create obligations for the given goal in the given environment. This may
    /// ultimately create any number of obligations.
    pub fn push_goal(&mut self, environment: &Arc<Environment>, goal: Goal) {
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
                                   ParameterKind::Lifetime(()) => {
                                       let lt = Lifetime::ForAll(new_environment.universe);
                                       ParameterKind::Lifetime(lt)
                                   }
                                   ParameterKind::Ty(()) =>
                                       ParameterKind::Ty(Ty::Apply(ApplicationTy {
                                           name: TypeName::ForAll(new_environment.universe),
                                           parameters: vec![]
                                       })),
                               }
                           })
                           .collect();
                let subgoal = subgoal.value.subst(&parameters);
                self.push_goal(&new_environment, subgoal);
            }
            Goal::Quantified(QuantifierKind::Exists, subgoal) => {
                let subgoal = self.instantiate_in(environment.universe,
                                                  subgoal.binders.iter().cloned(),
                                                  &subgoal.value);
                self.push_goal(environment, *subgoal);
            }
            Goal::Implies(wc, subgoal) => {
                let new_environment = &environment.add_clauses(wc);
                self.push_goal(new_environment, *subgoal);
            }
            Goal::And(subgoal1, subgoal2) => {
                self.push_goal(environment, *subgoal1);
                self.push_goal(environment, *subgoal2);
            }
            Goal::Not(subgoal) => {
                self.to_refute.push(InEnvironment::new(environment, *subgoal));
            }
            Goal::Leaf(wc) => {
                self.to_prove.push(InEnvironment::new(environment, wc));
            }
        }
    }

    fn solve_obligation(&mut self, wc: &InEnvironment<LeafGoal>)
                        -> Result<(Vec<ParameterInferenceVariable>, Solution)>
    {
        let canonicalized = self.infer.canonicalize(wc);
        let reduced_goal = canonicalized.quantified.into_reduced_goal();
        Ok((canonicalized.free_vars, self.solver.solve_reduced_goal(reduced_goal)?))
    }

    /// Provide all of the type inference variables created so far; used for REPL/debugging.
    pub fn ty_vars(&self) -> &[TyInferenceVariable] {
        self.infer.ty_vars()
    }

    /// Provide all of the type inference variables created so far; used for REPL/debugging.
    pub fn lifetime_vars(&self) -> &[LifetimeInferenceVariable] {
        self.infer.lifetime_vars()
    }

    /// Apply the subsitution `subst` to all the variables of `free_vars`
    /// (understood in deBruijn style), and add any lifetime constraints.
    fn apply_solution(&mut self,
                      free_vars: Vec<ParameterInferenceVariable>,
                      subst: Canonical<ConstrainedSubst>)
    {
        let Canonical { value, binders } = subst;
        let ConstrainedSubst { subst, constraints } = self.instantiate(binders, &value);

        debug!("fulfill::apply_solution: adding constraints {:?}", constraints);
        self.constraints.extend(constraints);

        // We use the empty environment for unification here because we're
        // really just doing a substitution on unconstrained variables, which is
        // guaranteed to succeed without generating any new constraints.
        let empty_env = &Environment::new();

        for (i, var) in free_vars.into_iter().enumerate() {
            match var {
                ParameterKind::Ty(ty) => {
                    let new_ty = subst.tys.get(&TyInferenceVariable::from_depth(i))
                        .expect("apply_solution failed to locate type variable in substitution");
                    self.unify(empty_env, &ty.to_ty(), &new_ty)
                        .expect("apply_solution failed to substitute");
                }
                ParameterKind::Lifetime(lt) => {
                    let new_lt = subst.lifetimes.get(&LifetimeInferenceVariable::from_depth(i))
                        .expect("apply_solution failed to find lifetime variable in substitution");
                    self.unify(empty_env, &lt.to_lifetime(), &new_lt)
                        .expect("apply_solution failed to substitute");
                }
            }
        }
    }

    fn fulfill_positive(&mut self) -> Result<Outcome> {
        debug_heading!("fulfill_positive(to_prove={:#?})", self.to_prove);

        // Try to solve all the obligations. We do this via a fixed-point
        // iteration. We try to solve each obligation in turn. Anything which is
        // successful, we drop; anything ambiguous, we retain in the
        // `obligations` array. This process is repeated so long as we are
        // learning new things about our inference state.
        let mut obligations = Vec::with_capacity(self.to_prove.len());
        let mut progress = true;

        while progress {
            progress = false;
            debug_heading!("start of round, {} obligations", self.to_prove.len());

            // Take the list of `obligations` to solve this round and replace it
            // with an empty vector. Iterate through each obligation to solve
            // and solve it if we can. If not (because of ambiguity), then push
            // it back onto `self.to_prove` for next round. Note that
            // `solve_one` may also push onto the `self.to_prove` list
            // directly.
            assert!(obligations.is_empty());
            while let Some(wc) = self.to_prove.pop() {
                let (free_vars, solution) = self.solve_obligation(&wc)?;

                if solution.has_definite() {
                    if let Some(constrained_subst) = solution.constrained_subst() {
                        self.apply_solution(free_vars, constrained_subst);
                        progress = true;
                    }
                }

                if solution.is_ambig() {
                    debug!("ambiguous result: {:?}", wc);
                    obligations.push(wc);
                }
            }

            self.to_prove.extend(obligations.drain(..));
            debug!("end of round, {} obligations left", self.to_prove.len());
        }

        // At the end of this process, `self.to_prove` should have
        // all of the ambiguous obligations, and `obligations` should
        // be empty.
        assert!(obligations.is_empty());

        if self.to_prove.is_empty() {
            Ok(Outcome::Complete)
        } else {
            Ok(Outcome::Incomplete)
        }
    }

    fn fulfill_negative(&mut self) -> Result<Outcome> {
        // Attempt to refute each negative obligations by in a fresh Fulfill
        for goal in self.to_refute.drain(..) {
            let canonicalized = self.infer.canonicalize(&goal);
            let goal = canonicalized.quantified.value;

            // Negation cannot be used to resolve existential variables, and do
            // not have a useful (for us) logial meaning when they contain
            // existential variables; treat negative goals with free variables
            // as ambiguous
            if !canonicalized.free_vars.is_empty() {
                return Ok(Outcome::Incomplete);
            }


            // Try to *solve* the goal in a fresh Fulfill
            let mut fulfill = Fulfill::new(self.solver);
            fulfill.push_goal(&goal.environment, goal.goal);

            // Negate the result
            if let Ok(solution) =  fulfill.solve(Substitution::empty()) {
                match solution {
                    Solution::Unique(_) => Err("refutation failed")?,
                    Solution::Ambig(_) => return Ok(Outcome::Incomplete),
                }
            }
        }

        Ok(Outcome::Complete)
    }

    /// Try to fulfill all pending obligations. The returned solution will
    /// transform `subst` substitution with the outcome of type inference by
    /// updating the replacements it provides.
    pub fn solve(mut self, subst: Substitution) -> Result<Solution> {
        // We first solve all positive (to_prove) goals, since these are the
        // only goals that can affect inference state or add new obligations.
        let positive = self.fulfill_positive()?;

        // Next we look at the goals we must refute; doing so cannot create new
        // obligations of any form
        let negative = self.fulfill_negative()?;

        if positive.is_complete() && negative.is_complete() {
            // No obligations remain, so we have definitively solved our goals,
            // and the current inference state is the unique way to solve them.

            let constraints = self.constraints.into_iter().collect();
            let constrained = self.infer.canonicalize(&ConstrainedSubst { subst, constraints });
            return Ok(Solution::Unique(constrained.quantified))
        }

        // Otherwise, we have (positive or negative) obligations remaining, but
        // haven't proved that it's *impossible* to satisfy out obligations. we
        // need to determine how to package up what we learned about type
        // inference as an ambiguous solution.

        if subst.is_trivial(&mut self.infer) {
            // In this case, we didn't learn *anything* definitively. So now, we
            // go one last time through the positive obligations, this time
            // applying even *tentative* inference suggestions, so that we can
            // yield these upwards as our own suggestions. In particular, we
            // yield up the first one we can find.

            while let Some(wc) = self.to_prove.pop() {
                let (free_vars, solution) = self.solve_obligation(&wc).unwrap();
                if let Some(constrained_subst) = solution.constrained_subst() {
                    self.apply_solution(free_vars, constrained_subst);
                    let subst = self.infer.canonicalize(&subst);
                    return Ok(Solution::Ambig(Guidance::Suggested(subst.quantified)));
                }
            }

            Ok(Solution::Ambig(Guidance::Unknown))
        } else {
            // While we failed to prove the goal, we still leared that
            // something had to hold. Here's an example where this happens:
            //
            // ```rust
            // trait Display {}
            // trait Debug {}
            // struct Foo<T> {}
            // struct Bar {}
            // struct Baz {}
            //
            // impl Display for Bar {}
            // impl Display for Baz {}
            //
            // impl<T> Debug for Foo<T> where T: Display {}
            // ```
            //
            // If we pose the goal `exists<T> { T: Debug }`, we can't say
            // for sure what `T` must be (it could be either `Foo<Bar>` or
            // `Foo<Baz>`, but we *can* say for sure that it must be of the
            // form `Foo<?0>`.
            let subst = self.infer.canonicalize(&subst);
            Ok(Solution::Ambig(Guidance::Definite(subst.quantified)))
        }
    }
}
