use errors::*;
use std::sync::Arc;

use super::*;
use solve::fulfill::Fulfill;
use std::cell::Cell;

thread_local! {
    // Default overflow depth which will be used in tests
    static OVERFLOW_DEPTH: Cell<usize> = Cell::new(10);
}

pub fn set_overflow_depth(overflow_depth: usize) {
    OVERFLOW_DEPTH.with(|depth| depth.set(overflow_depth));
}

pub fn get_overflow_depth() -> usize {
    OVERFLOW_DEPTH.with(|depth| depth.get())
}

/// We use a stack for detecting cycles. Each stack slot contains:
/// - a goal which is being processed
/// - a flag indicating the presence of a cycle during the processing of this goal
/// - in case a cycle has been found, the latest previous answer to the same goal
#[derive(Debug)]
struct StackSlot {
    goal: FullyReducedGoal,
    cycle: bool,
    answer: Option<Solution>,
}

/// For debugging purpose only: choose whether to apply a tabling strategy for cycles or
/// treat them as hard errors (the latter can possibly reduce debug output)
pub enum CycleStrategy {
    Tabling,
    Error,
}

/// A Solver is the basic context in which you can propose goals for a given
/// program. **All questions posed to the solver are in canonical, closed form,
/// so that each question is answered with effectively a "clean slate"**. This
/// allows for better caching, and simplifies management of the inference
/// context.
pub struct Solver {
    pub(super) program: Arc<ProgramEnvironment>,
    stack: Vec<StackSlot>,
    cycle_strategy: CycleStrategy,
    overflow_depth: usize,
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
    pub fn new(
        program: &Arc<ProgramEnvironment>,
        cycle_strategy: CycleStrategy,
        overflow_depth: usize
    ) -> Self {
        Solver {
            program: program.clone(),
            stack: vec![],
            cycle_strategy,
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
        debug_heading!("solve_closed_goal(goal={:?})", goal);

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

    /// This ought to be the *True* entry point.
    pub fn solve_canonical_goal(&mut self,
                                canonical_goal: &Canonical<InEnvironment<Goal>>)
                                -> Result<Solution> {
        let mut fulfill = Fulfill::new(self);

        let goal = fulfill.instantiate(canonical_goal.binders.iter().cloned(),
                                       &canonical_goal.value);

        // We use this somewhat hacky approach to get our hands on the
        // instantiated variables after instantiating the canonical
        // goal. This substitution is only used for REPL/debugging
        // purposes anyway; in rustc, the top-level interaction would
        // happen by manipulating a Fulfill more directly.
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

        fulfill.push_goal(&goal.environment, goal.goal);
        fulfill.solve(subst)
    }

    /// Attempt to solve a goal that has been fully broken down into leaf form
    /// and canonicalized. This is where the action really happens, and is the
    /// place where we would perform caching in rustc (and may eventually do in Chalk).
    pub fn solve_reduced_goal(&mut self, goal: FullyReducedGoal) -> Result<Solution> {
        debug_heading!("Solver::solve({:?})", goal);

        if self.stack.len() > self.overflow_depth {
            panic!("overflow depth reached");
        }

        // The goal was already on the stack: we found a cycle.
        if let Some(index) = self.stack.iter().position(|s| { s.goal == goal }) {

            // If we are facing a goal of the form `?0: AutoTrait`, we apply coinductive semantics:
            // if all the components of the cycle also have coinductive semantics, we accept
            // the cycle `(?0: AutoTrait) :- ... :- (?0: AutoTrait)` as an infinite proof for
            // `?0: AutoTrait` and we do not perform any substitution.
            if self.stack.iter()
                         .skip(index)
                         .map(|s| &s.goal)
                         .chain(Some(&goal))
                         .all(|g| g.is_coinductive(&*self.program))
            {
                let value = ConstrainedSubst {
                    subst: Substitution::empty(),
                    constraints: vec![],
                };
                debug!("applying coinductive semantics");
                return Ok(Solution::Unique(Canonical { value, binders: goal.into_binders() }));
            }

            // Else we indicate that we found a cycle by setting `slot.cycle = true`.
            // If there is no cached answer, we can't make any more progress and return `Err`.
            // If there is one, use this answer.
            let slot = &mut self.stack[index];
            slot.cycle = true;
            debug!("cycle detected: previous solution {:?}", slot.answer);
            return slot.answer.clone().ok_or("cycle".into());
        }

        // We start with `answer = None` and try to solve the goal. At the end of the iteration,
        // `answer` will be updated with the result of the solving process. If we detect a cycle
        // during the solving process, we cache `answer` and try to solve the goal again. We repeat
        // until we reach a fixed point for `answer`.
        // Considering the partial order:
        // - None < Some(Unique) < Some(Ambiguous)
        // - None < Some(CannotProve)
        // the function which maps the loop iteration to `answer` is a nondecreasing function
        // so this function will eventually be constant and the loop terminates.
        let mut answer = None;
        loop {
            self.stack.push(StackSlot {
                goal: goal.clone(),
                cycle: false,
                answer: answer.clone(),
            });

            debug!("Solver::solve: new loop iteration");
            let result = match goal.clone() {
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

                    let env_clauses = value.environment.clauses.iter()
                        .cloned()
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
            debug!("Solver::solve: loop iteration result = {:?}", result);

            let slot = self.stack.pop().unwrap();
            match self.cycle_strategy {
                CycleStrategy::Tabling if slot.cycle => {
                    let actual_answer = result.as_ref().ok().map(|s| s.clone());
                    let fixed_point = answer == actual_answer;

                    // If we reach a fixed point, we can break.
                    // If the answer is `Ambig`, then we know that we already have multiple
                    // solutions, and we *must* break because an `Ambig` solution may not perform
                    // any unification and thus fail to correctly reach a fixed point. See test
                    // `multiple_ambiguous_cycles`.
                    match (fixed_point, &actual_answer) {
                        (_, &Some(Solution::Ambig(_))) | (true, _) =>
                            return result,
                        _ => ()
                    };

                    answer = actual_answer;
                }
                _ => return result,
            };
        }
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
        let ProgramClauseImplication { consequence, conditions } =
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
