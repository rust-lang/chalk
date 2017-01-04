use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
use solve::infer::Quantified;
use solve::implemented::Implemented;
use std::sync::Arc;

use super::*;

pub struct Solver {
    pub(super) program: Arc<Program>,
}

impl Solver {
    pub fn solve(&mut self,
                 wc: Quantified<InEnvironment<WhereClause>>)
                 -> Result<Solution<Quantified<InEnvironment<WhereClause>>>> {
        let Quantified { value: InEnvironment { environment, goal: wc }, binders } = wc;
        match wc {
            WhereClause::Implemented(trait_ref) => {
                let q = Quantified {
                    value: InEnvironment::new(&environment, trait_ref),
                    binders: binders,
                };
                Implemented::new(self, q).solve().map(|soln| {
                    soln.map_goal(|refined_goal| {
                        refined_goal.map(|in_env| in_env.map_goal(WhereClause::Implemented))
                    })
                })
            }
            WhereClause::NormalizeTo(normalize_to) => unimplemented!(),
        }
    }
}
