use cast::Cast;
use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::implemented::Implemented;
use solve::infer::InferenceTable;
use std::sync::Arc;

use super::*;

pub struct Solver {
    pub(super) program: Arc<Program>,

    stack: Vec<Quantified<InEnvironment<WhereClause>>>,
}

impl Solver {
    pub fn new(program: &Arc<Program>) -> Self {
        Solver { program: program.clone(), stack: vec![] }
    }

    /// Tries to solve one **closed** where-clause `wc` (in the given
    /// environment).
    pub fn solve(&mut self,
                 wc_env: Quantified<InEnvironment<WhereClause>>)
                 -> Result<Solution<Quantified<InEnvironment<WhereClause>>>> {
        debug!("Solver::solve({:?})", wc_env);

        if self.stack.contains(&wc_env) {
            // Recursive invocation
            debug!("solve: {:?} already on the stack", wc_env);
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
            WhereClause::NormalizeTo(_normalize_to) => unimplemented!(),
        };

        self.stack.pop().unwrap();

        result
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
