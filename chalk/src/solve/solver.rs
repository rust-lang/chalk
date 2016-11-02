#![allow(unused_variables)]

use formula::*;
use solve::*;
use std::sync::Arc;

pub struct Solver {
    infer: InferenceTable,
    root_goal: Goal<Leaf>,
    solutions: Vec<Goal<Leaf>>,
    obligations: Vec<Obligation>,
}

impl Solver {
    pub fn new(root_environment: Arc<Environment>,
               root_goal: Goal<Leaf>)
               -> Self {
        Solver {
            infer: InferenceTable::new(),
            root_goal: root_goal.clone(),
            solutions: vec![],
            obligations: vec![Obligation::new(root_environment, root_goal)],
        }
    }

    fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        self.infer.new_variable(ui)
    }

    fn canonicalize(&mut self, goal: &Goal<Leaf>) -> Goal<Leaf> {
        unimplemented!()
    }

    fn probe<F, R>(&mut self, op: F) -> R
        where F: FnOnce(&mut Self) -> R
    {
        let snapshot = self.infer.snapshot();
        let obligations = self.obligations.clone();
        let result = op(self);
        self.infer.rollback_to(snapshot);
        self.obligations = obligations;
        result
    }

    fn run(&mut self) {
        while let Some(obligation) = self.obligations.pop() {
            match self.solve_obligation(obligation) {
                Ok(()) => { }
                Err(()) => { return; }
            }
        }

        let goal = self.root_goal.clone();
        let goal = self.canonicalize(&goal);
        self.solutions.push(goal);
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
                for goal in goals {
                    self.probe(|this| {
                        this.obligations.push(Obligation {
                            environment: environment.clone(),
                            goal: goal.clone(),
                        });
                        this.run();
                    });
                }
                Err(()) // signal to the surrounding `run()` task that we took over =)
            }
            GoalKind::Exists(ref quant) => unimplemented!(),
            GoalKind::Implication(ref quant, ref goal) => unimplemented!(),
            GoalKind::ForAll(ref quant) => unimplemented!(),
        }
    }
}
