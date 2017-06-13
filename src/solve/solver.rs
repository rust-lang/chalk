use errors::*;
use std::sync::Arc;

use super::*;
use solve::fulfill::Fulfill;

/// A Solver is the basic context in which you can propose goals for a given
/// program. **All questions posed to the solver are in canonical, closed form,
/// so that each question is answered with effectively a "clean slate"**. This
/// allows for better caching, and simplifies management of the inference
/// context. Solvers do, however, maintain a stack of questions being posed, so
/// as to avoid unbounded search.
pub struct Solver {
    pub(super) program: Arc<ProgramEnvironment>,
    overflow_depth: usize,
    stack: Vec<FullyReducedGoal>,
}

/// An extension trait for merging `Result`s
trait MergeWith<T> {
    fn merge_with<F>(self, other: Self, f: F) -> Self where F: FnOnce(T, T) -> T;
}

impl<T> MergeWith<T> for Result<T> {
    fn merge_with<F>(self: Result<T>, other: Result<T>, f: F) -> Result<T>
        where F: FnOnce(T, T) -> T
    {
        match (self, other) {
            (Err(_), Ok(v)) |
            (Ok(v), Err(_)) => Ok(v),
            (Ok(v1), Ok(v2)) => Ok(f(v1, v2)),
            (Err(_), Err(e)) => Err(e)
        }
    }
}

impl Solver {
    pub fn new(program: &Arc<ProgramEnvironment>, overflow_depth: usize) -> Self {
        Solver {
            program: program.clone(),
            stack: vec![],
            overflow_depth,
        }
    }

    /// Attempt to solve a *closed* goal. The substitution returned in the
    /// solution will be for the fully decomposed goal. For example, given the
    /// program
    ///
    /// ```ignore
    /// struct u8 { }
    /// struct SomeType<T> { }
    /// trait Foo<T> { }
    /// impl<U> Foo<u8> for SomeType<U> { }
    /// ```
    ///
    /// and the goal `exists<V> { forall<U> { SomeType<U>: Foo<V> } }`, a unique
    /// solution is produced with substitution `?0 := u8`. The `?0` is drawn
    /// from the number of the instantiated existential.
    pub fn solve_closed_goal(&mut self, goal: InEnvironment<Goal>) -> Result<Solution> {
        let mut fulfill = Fulfill::new(self);
        fulfill.push_goal(&goal.environment, goal.goal);

        // We use this somewhat hacky approach to get our hands on the
        // instantiated variables after pushing our initial goal. This
        // substitution is only used for REPL/debugging purposes anyway; in
        // rustc, the top-level interaction would happen by manipulating a
        // Fulfill more directly.
        let subst = Substitution {
            tys: fulfill
                .ty_vars()
                .iter()
                .map(|t| (*t, t.to_ty()))
                .collect(),
            lifetimes: fulfill
                .lifetime_vars()
                .iter()
                .map(|lt| (*lt, lt.to_lifetime()))
                .collect(),
        };

        fulfill.solve(subst)
    }

    /// Attempt to solve a goal that has been fully broken down into leaf form
    /// and canonicalized. This is where the action really happens, and is the
    /// place where we would perform caching in rustc (and may eventually do in Chalk).
    pub fn solve_reduced_goal(&mut self, goal: FullyReducedGoal) -> Result<Solution> {
        debug_heading!("Solver::solve({:?})", goal);

        // First we cut off runaway recursion
        if self.stack.contains(&goal) || self.stack.len() > self.overflow_depth {
            // Recursive invocation or overflow
            debug!(
                "solve: {:?} already on the stack or overflowed max depth",
                goal
            );
            return Ok(Solution::Ambig(Guidance::Unknown));
        }
        self.stack.push(goal.clone());

        let result = match goal {
            FullyReducedGoal::EqGoal(g) => {
                // Equality goals are understood "natively" by the logic, via unification:
                self.solve_via_unification(g)
            }
            FullyReducedGoal::DomainGoal(Canonical { value, binders }) => {
                // "Domain" goals (i.e., leaf goals that are Rust-specific) are
                // always solved via some form of implication. We can either
                // apply assumptions from our environment (i.e. where clauses),
                // or from the lowered program, which includes fallback
                // clauses. We try each approach in turn:

                let env_clauses = value
                    .environment
                    .elaborated_clauses(&self.program)
                    .map(DomainGoal::into_program_clause);
                let env_solution = self.solve_from_clauses(&binders, &value, env_clauses);

                let prog_clauses: Vec<_> = self.program.program_clauses.iter()
                    .cloned()
                    .filter(|clause| !clause.fallback_clause)
                    .collect();
                let prog_solution = self.solve_from_clauses(&binders, &value, prog_clauses);

                // These fallback clauses are used when we're sure we'll never
                // reach Unique via another route
                let fallback: Vec<_> = self.program.program_clauses.iter()
                    .cloned()
                    .filter(|clause| clause.fallback_clause)
                    .collect();
                let fallback_solution = self.solve_from_clauses(&binders, &value, fallback);

                // Now that we have all the outcomes, we attempt to combine
                // them. Here, we apply a heuristic (also found in rustc): if we
                // have possible solutions via both the environment *and* the
                // program, we favor the environment; this only impacts type
                // inference. The idea is that the assumptions you've explicitly
                // made in a given context are more likely to be relevant than
                // general `impl`s.

                env_solution
                    .merge_with(prog_solution, |env, prog| env.favor_over(prog))
                    .merge_with(fallback_solution, |merged, fallback| merged.fallback_to(fallback))
            }
        };

        self.stack.pop().unwrap();

        debug!("Solver::solve: result={:?}", result);
        result
    }

    fn solve_via_unification(
        &mut self,
        goal: Canonical<InEnvironment<EqGoal>>,
    ) -> Result<Solution> {
        let mut fulfill = Fulfill::new(self);
        let Canonical { value, binders } = goal;
        let subst = Substitution::from_binders(&binders);
        let (InEnvironment { environment, goal }, subst) =
            fulfill.instantiate(binders, &(value, subst));

        fulfill.unify(&environment, &goal.a, &goal.b)?;
        fulfill.solve(subst)
    }

    /// See whether we can solve a goal by implication on any of the given
    /// clauses. If multiple such solutions are possible, we attempt to combine
    /// them.
    fn solve_from_clauses<C>(
        &mut self,
        binders: &[ParameterKind<UniverseIndex>],
        goal: &InEnvironment<DomainGoal>,
        clauses: C
    ) -> Result<Solution>
    where
        C: IntoIterator<Item = ProgramClause>,
    {
        let mut cur_solution = None;
        for ProgramClause { implication, .. } in clauses {
            debug_heading!("clause={:?}", implication);

            let res = self.solve_via_implication(binders, goal.clone(), implication);
            if let Ok(solution) = res {
                debug!("ok: solution={:?}", solution);
                cur_solution = Some(
                    match cur_solution {
                        None => solution,
                        Some(cur) => solution.combine(cur),
                    },
                );
            } else {
                debug!("error");
            }
        }
        cur_solution.ok_or("no applicable candidates".into())
    }

    /// Modus ponens! That is: try to apply an implication by proving its premises.
    fn solve_via_implication(
        &mut self,
        binders: &[ParameterKind<UniverseIndex>],
        goal: InEnvironment<DomainGoal>,
        clause: Binders<ProgramClauseImplication>
    ) -> Result<Solution> {
        let mut fulfill = Fulfill::new(self);
        let subst = Substitution::from_binders(&binders);
        let (goal, (clause, subst)) =
            fulfill.instantiate(binders.iter().cloned(), &(goal, (clause, subst)));
        let ProgramClauseImplication { consequence, conditions} =
            fulfill.instantiate_in(goal.environment.universe, clause.binders, &clause.value);

        fulfill.unify(&goal.environment, &goal.goal, &consequence)?;

        // if so, toss in all of its premises
        for condition in conditions {
            fulfill.push_goal(&goal.environment, condition);
        }

        // and then try to solve
        fulfill.solve(subst)
    }
}
