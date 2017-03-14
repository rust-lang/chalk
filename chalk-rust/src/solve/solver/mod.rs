use cast::Cast;
use errors::*;
use solve::environment::InEnvironment;
use solve::match_program_clause::MatchProgramClause;
use solve::normalize::SolveNormalize;
use solve::not_unify::SolveNotUnify;
use solve::implemented::Implemented;
use solve::unify::SolveUnify;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

use super::*;

pub struct Solver {
    pub(super) program: Arc<ProgramEnvironment>,
    overflow_depth: usize,
    stack: Vec<Query<InEnvironment<WhereClauseGoal>>>,
}

impl Solver {
    pub fn new(program: &Arc<ProgramEnvironment>, overflow_depth: usize) -> Self {
        Solver { program: program.clone(), stack: vec![], overflow_depth, }
    }

    /// Tries to solve one **closed** where-clause `wc` (in the given
    /// environment).
    pub fn solve(&mut self,
                 wc_env: Query<InEnvironment<WhereClauseGoal>>)
                 -> Result<Solution<InEnvironment<WhereClauseGoal>>> {
        debug_heading!("Solver::solve({:?})", wc_env);

        if self.stack.contains(&wc_env) || self.stack.len() > self.overflow_depth {
            // Recursive invocation or overflow
            debug!("solve: {:?} already on the stack or overflowed max depth", wc_env);
            return Ok(Solution {
                successful: Successful::Maybe,
                refined_goal: wc_env.map(|w| Constrained { value: w, constraints: vec![] })
            });
        }

        self.stack.push(wc_env.clone());

        let Query { value: InEnvironment { environment, goal: wc }, binders } = wc_env;

        let result = match wc {
            WhereClauseGoal::Implemented(trait_ref) => {
                let q = Query {
                    value: InEnvironment::new(&environment, trait_ref),
                    binders: binders,
                };
                Implemented::new(self, q).solve().cast()
            }
            WhereClauseGoal::Normalize(normalize_to) => {
                let q = Query {
                    value: InEnvironment::new(&environment, normalize_to),
                    binders: binders,
                };
                SolveNormalize::new(self, q).solve().cast()
            }
            WhereClauseGoal::UnifyTys(unify) => {
                let q = Query {
                    value: InEnvironment::new(&environment, unify),
                    binders: binders,
                };
                SolveUnify::new(self, q).solve().cast()
            }
            WhereClauseGoal::NotUnifyTys(not_unify) => {
                let q = Query {
                    value: InEnvironment::new(&environment, not_unify),
                    binders: binders,
                };
                SolveNotUnify::new(self, q).solve().cast()
            }
            WhereClauseGoal::UnifyKrates(unify) => {
                let q = Query {
                    value: InEnvironment::new(&environment, unify),
                    binders: binders,
                };
                SolveUnify::new(self, q).solve().cast()
            }
            WhereClauseGoal::NotImplemented(_) |
            WhereClauseGoal::NotNormalize(_) |
            WhereClauseGoal::TyLocalTo(_) |
            WhereClauseGoal::WellFormed(_) => {
                // Currently, we don't allow `LocalTo` or `WF` types
                // into the environment, there we just have to search
                // for program clauses.
                let program = self.program.clone();
                let q = Query {
                    value: InEnvironment::new(&environment, wc),
                    binders: binders
                }; // reconstruct `wc_env`
                self.solve_any(program.program_clauses.iter(), &q, |this, program_clause| {
                    MatchProgramClause::new(this, &q, &program_clause).solve()
                })
            }
        };

        self.stack.pop().unwrap();

        debug!("Solver::solve: result={:?}", result);

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
                               start_goal: &Query<R>,
                               mut evaluate_possibility: E)
                               -> Result<Solution<R>>
        where Ps: IntoIterator<Item = P>,
              E: FnMut(&mut Solver, P) -> Result<Solution<R>>,
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
                    if {
                        solution.refined_goal.binders == start_goal.binders &&
                        solution.refined_goal.value.value == start_goal.value &&
                            solution.refined_goal.value.constraints.is_empty()
                    } {
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
            refined_goal: start_goal.clone().map(|g| Constrained { value: g, constraints: vec![] }),
        });
    }
}
