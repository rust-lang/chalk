use ena::unify::UnificationTable;
use formula::*;
use solve::*;
use std::mem;
use std::sync::Arc;

pub struct Solver {
    unify: UnificationTable<InferenceVariable>,
    root_environment: Arc<Environment>,
    solutions: Vec<Goal<Leaf>>,
    obligations: Vec<Obligation>,
}

impl Solver {
    pub fn new(clauses: Vec<Clause<Leaf>>) -> Self {
        Solver {
            unify: UnificationTable::new(),
            root_environment: Arc::new(Environment::new(None, clauses)),
            solutions: vec![],
            obligations: vec![],
        }
    }

    pub fn solve_goal(&mut self, goal: Goal<Leaf>) -> Result<Vec<Goal<Leaf>>, ()> {
        assert!(self.solutions.is_empty());
        self.obligations.push(Obligation::new(self.root_environment.clone(), goal));
        self.run()?;
        Ok(mem::replace(&mut self.solutions, vec![]))
    }

    fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        self.unify.new_key(ui)
    }

    fn universe_index(&mut self, v: InferenceVariable) -> UniverseIndex {
        self.unify.probe_value(v)
    }

    fn probe<F, R>(&mut self, op: F) -> R
        where F: FnOnce(&mut Self) -> R
    {
        let snapshot = self.unify.snapshot();
        let result = op(self);
        self.unify.rollback_to(snapshot);
        result
    }

    fn run(&mut self) -> Result<(), ()> {
        while let Some(obligation) = self.obligations.pop() {
            self.solve_obligation(obligation)?;
        }
        Ok(())
    }

    fn solve_obligation(&mut self, obligation: Obligation) -> Result<(), ()> {
        let Obligation { environment, goal } = obligation;
        match goal.kind {
            GoalKind::True => Ok(()),
            GoalKind::Leaf(ref leaf) => unimplemented!(),
            GoalKind::And(ref goals) => {
                self.obligations.extend(goals.iter()
                    .map(|goal| {
                        Obligation {
                            environment: environment.clone(),
                            goal: goal.clone(),
                        }
                    }));
                Ok(())
            }
            GoalKind::Or(ref goals) => {
                unimplemented!()
            }
            GoalKind::Exists(ref quant) => unimplemented!(),
            GoalKind::Implication(ref quant, ref goal) => unimplemented!(),
            GoalKind::ForAll(ref quant) => unimplemented!(),
        }
    }
}
