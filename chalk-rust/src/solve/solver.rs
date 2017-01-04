use errors::*;
use ir::*;
use solve::infer::Quantified;

use super::*;

pub struct Solver {
}

impl Solver {
    pub fn solve_wc(&mut self, wc: Quantified<WhereClause>) -> Result<Successful> {
        unimplemented!()
    }

    pub fn solve_all<'a, WC>(&mut self, wcs: WC) -> Result<Successful>
        where WC: IntoIterator<Item=Quantified<WhereClause>>
    {
        let mut successful = Successful::Yes;
        for wc in wcs {
            match self.solve_wc(wc)? {
                Successful::Yes => { }
                Successful::Maybe => successful = Successful::Maybe,
            }
        }
        Ok(successful)
    }
}
