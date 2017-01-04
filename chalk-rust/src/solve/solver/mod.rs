use cast::Cast;
use errors::*;
use ir::*;
use solve::environment::InEnvironment;
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
                Implemented::new(self, q).solve().cast()
            }
            WhereClause::NormalizeTo(_normalize_to) => unimplemented!(),
        }
    }
}
