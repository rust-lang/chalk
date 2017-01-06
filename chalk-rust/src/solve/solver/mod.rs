use cast::Cast;
use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::normalize::SolveNormalize;
use solve::implemented::Implemented;
use solve::infer::InferenceTable;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use super::*;

pub struct Solver {
    pub(super) program: Arc<Program>,
    overflow_depth: usize,
    stack: Vec<Quantified<InEnvironment<WhereClause>>>,
}

impl Solver {
    pub fn new(program: &Arc<Program>, overflow_depth: usize) -> Self {
        Solver { program: program.clone(), stack: vec![], overflow_depth, }
    }

    /// Tries to solve one **closed** where-clause `wc` (in the given
    /// environment).
    pub fn solve(&mut self,
                 wc_env: Quantified<InEnvironment<WhereClause>>)
                 -> Result<Solution<Quantified<InEnvironment<WhereClause>>>> {
        debug!("Solver::solve({:?})", wc_env);

        if self.stack.contains(&wc_env) || self.stack.len() > self.overflow_depth {
            // Recursive invocation or overflow
            debug!("solve: {:?} already on the stack or overflowed max depth", wc_env);
            return Ok(Solution {
                successful: Successful::Maybe,
                refined_goal: wc_env,
            });
        }

        self.stack.push(wc_env.clone());

        let Quantified { value: InEnvironment { environment, goal: wc }, binders } = wc_env;
        let result = match wc {
            WhereClause::Implemented(trait_ref) => {
                let q = Quantified {
                    value: InEnvironment::new(&environment, trait_ref),
                    binders: binders,
                };
                Implemented::new(self, q).solve().cast()
            }
            WhereClause::Normalize(normalize_to) => {
                let q = Quantified {
                    value: InEnvironment::new(&environment, normalize_to),
                    binders: binders,
                };
                SolveNormalize::new(self, q).solve().cast()
            }
        };

        self.stack.pop().unwrap();

        result
    }

    /// Given a closed goal `start_goal`, and a number of possible
    /// ways to solve it (`possibilities`), invokes
    /// `evaluate_possibility` to evaluate each possibility in
    /// turn. Collates the results and, if there is an unambigious
    /// result, returns it.
    ///
    /// In general, an unambiguous result can result either because we
    /// found only a single possibility that did not yield an error or
    /// because multiple possibilities resulted in the same solution.
    pub fn solve_any<P,Ps,E,R>(&mut self,
                               possibilities: Ps,
                               start_goal: &Quantified<R>,
                               mut evaluate_possibility: E)
                               -> Result<Solution<Quantified<R>>>
        where Ps: IntoIterator<Item = P>,
              E: FnMut(&mut Solver, P) -> Result<Solution<Quantified<R>>>,
              R: Clone + Hash + Eq
    {
        // For each impl, recursively apply it. Note that all we need
        // to verify is that `T: Foo` **is implemented**. We don't
        // actually need to know *which impl* implified with.
        let mut candidates = HashSet::new();
        for possibility in possibilities {
            if let Ok(solution) = evaluate_possibility(self, possibility) {
                // If we found an impl which definitively applies
                // **without unifying anything in the goal**, then we
                // know that the type is indeed implemented (though
                // there may be other impls which also apply, because
                // of specialization).
                //
                // If the impl **does unify things in the goal**, then
                // it only applies **conditionally**, and we have to
                // see what other impls apply. If this is indeed the
                // only applicable one, then we can opt to use it (and
                // this implies that those variables can be unified on
                // the other side, since its the only way to ensure
                // that the trait is implemented). But if there are
                // multiple impls, perhaps with distinct unifications,
                // then we have to return an ambiguous result.
                if let Successful::Yes = solution.successful {
                    if solution.refined_goal == *start_goal {
                        return Ok(solution);
                    }
                }

                candidates.insert(solution);
            }
        }

        if candidates.len() == 0 {
            bail!("no applicable candidates")
        }

        if candidates.len() == 1 {
            return Ok(candidates.into_iter().next().unwrap());
        }

        // There are multiple candidates and they don't agree about
        // what we can infer thus far. Return an ambiguous
        // result. This actually isn't as precise as it could be, in
        // two ways:
        //
        // a. It might be that while there are multiple distinct
        //    candidates, they all agree about *some things*. To be
        //    maximally precise, we would compute the intersection of
        //    what they agree on. It's not clear though that this is
        //    actually what we want Rust's inference to do, and it's
        //    certainly not what it does today.
        // b. There might also be an ambiguous candidate and a successful
        //    candidate, both with the same refined-goal. In that case,
        //    we could probably claim success, since if the conditions of the
        //    ambiguous candidate were met, we now the success would apply.
        //    Example: `?0: Clone` yields ambiguous candidate `Option<?0>: Clone`
        //    and successful candidate `Option<?0>: Clone`.
        //
        // But you get the idea.
        return Ok(Solution {
            successful: Successful::Maybe,
            refined_goal: start_goal.clone(),
        });
    }

    /// Try to solve all of `where_clauses`, which may contain
    /// inference variables registered in the table `infer`. This can
    /// have side-effects on the inference state (regardless of
    /// whether it returns success, failure, or ambiguity). But, in
    /// all cases, the side-effects are only things that must be true
    /// for `where_clauses` to be true.
    pub fn solve_all(&mut self,
                     infer: &mut InferenceTable,
                     mut where_clauses: Vec<InEnvironment<WhereClause>>)
                     -> Result<Successful> {
        debug!("solve_all(where_clauses={:#?})", where_clauses);

        // Try to solve all the where-clauses. We do this via a
        // fixed-point iteration. We try to solve each where-clause in
        // turn. Anything which is successful, we drop; anything
        // ambiguous, we retain in the `where_clauses` array. This
        // process is repeated so long as we are learning new things
        // about our inference state.
        let mut retained = Vec::with_capacity(where_clauses.len());
        let mut progress = true;
        while progress {
            progress = false;

            for wc in where_clauses.drain(..) {
                match self.solve_one(infer, &wc, &mut progress)? {
                    Successful::Yes => (),
                    Successful::Maybe => retained.push(wc),
                }
            }

            where_clauses.extend(retained.drain(..));
        }

        // If we still have ambiguous where-clauses, then we have an
        // ambiguous overall result.
        if where_clauses.is_empty() {
            Ok(Successful::Yes)
        } else {
            Ok(Successful::Maybe)
        }
    }

    fn solve_one(&mut self,
                 infer: &mut InferenceTable,
                 wc: &InEnvironment<WhereClause>,
                 inference_progress: &mut bool)
                 -> Result<Successful> {
        let quantified_wc = infer.quantify(&wc);
        let solution = self.solve(quantified_wc.clone())?;

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
        if solution.refined_goal != quantified_wc {
            let refined_goal =
                infer.instantiate(wc.environment.universe, &solution.refined_goal.value);
            infer.unify(wc, &refined_goal)?;
            *inference_progress = true;
        }

        Ok(solution.successful)
    }
}
