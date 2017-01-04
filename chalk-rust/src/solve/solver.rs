use errors::*;
use ir::*;
use solve::environment::Environment;
use solve::infer::Quantified;
use std::sync::Arc;

use super::*;

pub struct Solver {
    pub(super) program: Arc<Program>,
}

impl Solver {
    pub fn solve_wc(&mut self,
                    wc: Quantified<(Arc<Environment>, WhereClause)>)
                    -> Result<Successful> {
        unimplemented!()
    }

    pub fn solve_all<'a, WC>(&mut self, wcs: WC) -> Result<Successful>
        where WC: IntoIterator<Item = Quantified<(Arc<Environment>, WhereClause)>>
    {
        let mut successful = Successful::Yes;
        for wc in wcs {
            match self.solve_wc(wc)? {
                Successful::Yes => {}
                Successful::Maybe => successful = Successful::Maybe,
            }
        }
        Ok(successful)
    }
}
