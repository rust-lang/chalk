use errors::*;
use ir::*;
use solve::environment::{Environment, InEnvironment};
use solve::infer::{InferenceTable, InferenceVariable, UniverseIndex};
use solve::solver::Solver;
use solve::Successful;
use std::sync::Arc;

pub struct Prove<'s> {
    solver: &'s mut Solver,
    infer: InferenceTable,
    goals: Vec<InEnvironment<WhereClause>>,
}

enum Binding {
    ForAll(UniverseIndex),
    Exists(InferenceVariable),
}

impl<'s> Prove<'s> {
    pub fn new(solver: &'s mut Solver, goal: Box<Goal>) -> Self {
        let mut prove = Prove { solver, infer: InferenceTable::new(), goals: vec![] };
        let environment = &Environment::new();
        prove.decompose(goal, environment, &mut vec![]);
        prove
    }

    fn decompose(&mut self,
                 goal: Box<Goal>,
                 environment: &Arc<Environment>,
                 bindings: &mut Vec<Binding>) {
        let goal = *goal;
        match goal {
            Goal::ForAll(num_binders, subgoal) => {
                let mut new_environment = environment.clone();
                for _ in 0 .. num_binders {
                    new_environment = new_environment.new_universe();
                    bindings.push(Binding::ForAll(new_environment.universe));
                }
                self.decompose(subgoal, &new_environment, bindings);
                for _ in 0 .. num_binders {
                    bindings.pop();
                }
            }
            Goal::Exists(num_binders, subgoal) => {
                for _ in 0 .. num_binders {
                    bindings.push(Binding::Exists(self.infer.new_variable(environment.universe)));
                }
                self.decompose(subgoal, environment, bindings);
                for _ in 0 .. num_binders {
                    bindings.pop().unwrap();
                }
            }
            Goal::Implies(wc, subgoal) => {
                let new_environment = &environment.add_clauses(wc);
                self.decompose(subgoal, new_environment, bindings);
            }
            Goal::And(subgoal1, subgoal2) => {
                self.decompose(subgoal1, environment, bindings);
                self.decompose(subgoal2, environment, bindings);
            }
            Goal::Leaf(wc) => {
                // FIXME need to apply substitution here
                self.goals.push(InEnvironment::new(environment, wc));
            }
        }
    }

    pub fn solve(mut self) -> Result<Successful> {
        self.solver.solve_all(&mut self.infer, self.goals)
    }
}
