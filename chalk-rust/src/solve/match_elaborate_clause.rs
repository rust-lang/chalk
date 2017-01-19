use cast::Cast;
use errors::*;
use fold::{Fold, Subst};
use ir::*;
use solve::Solution;
use solve::match_clause::MatchClause;
use solve::environment::InEnvironment;
use solve::solver::Solver;
use std::hash::Hash;

/// Example:
///
/// - `G` is `T: PartialEq`.
/// - The trait `Eq` is defined with `where Self: PartialEq`, which applies to `G`.
/// - `T: Eq` is in the environment.
pub struct MatchElaborateClause<'s, G: 's> {
    solver: &'s mut Solver,
    env_goal: &'s Quantified<InEnvironment<G>>,
    clause: &'s WhereClause,
}

impl<'s, G> MatchElaborateClause<'s, G>
    where G: Clone + Cast<WhereClause> + Fold<Result = G> + Hash + Eq
{
    pub fn new(solver: &'s mut Solver,
               env_goal: &'s Quantified<InEnvironment<G>>,
               clause: &'s WhereClause)
               -> Self {
        MatchElaborateClause { solver, env_goal, clause }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<G>>> {
        let program = self.solver.program.clone();

        let trait_ref = match *self.clause {
            WhereClause::Implemented(ref trait_ref) => trait_ref,
            WhereClause::Normalize(..) => {
                bail!("elaborating normalization not implemented")
            }
        };

        let trait_data = &program.trait_data[&trait_ref.trait_id];
        let where_clauses = Subst::apply(&trait_ref.parameters, &trait_data.where_clauses);
        let env_goal = self.env_goal;
        self.solver.solve_any(where_clauses.iter(), env_goal, |solver, wc| {
            // FIXME not transitive, not really a good solution
            MatchClause::new(solver, env_goal, wc).solve()
        })
    }
}
