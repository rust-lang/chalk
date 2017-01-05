use errors::*;
use ir::*;
use solve::environment::InEnvironment;
use solve::implemented_with::ImplementedWith;
use solve::solver::Solver;
use solve::{Solution, Successful};
use std::collections::HashSet;

pub struct ImplementedEnv<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    environment: Arc<Environment>,
    goal: TraitRef,
}

impl<'s> ImplementedEnv<'s> {
    pub fn new(solver: &'s mut Solver,
               q: Quantified<InEnvironment<TraitRef>>)
               -> Self {
        let InEnvironment { environment, goal } = q.value;
        let infer = InferenceTable::new_with_vars(q.binders, environment.universe);
        ImplementedEnv { solver, infer, environment, goal }
    }

    pub fn solve(&mut self) -> Result<Solution<Quantified<InEnvironment<TraitRef>>>> {
        unimplemented!()
    }
}
