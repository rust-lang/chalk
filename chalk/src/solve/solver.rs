#![allow(unused_variables)]

use infer::*;
use formula::*;
use solve::*;
use subst::Subst;
use std::sync::Arc;

pub struct Solver {
    infer: InferenceTable,
    root_goal: Goal<Leaf>,
    solutions: Vec<Goal<Leaf>>,
    obligations: Vec<Obligation>,
}

impl Solver {
    pub fn new(root_environment: Arc<Environment>, root_goal: Goal<Leaf>) -> Self {
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
        // FIXME -- this meant to replace unbound variables like ?F
        // with a `_`, but that is not a variant of leaf (and should
        // not be). Should be able to fix this without extending leaf
        // but might need to generalize folder trait.
        self.infer.normalize_deep(goal)
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
            GoalKind::And(ref g1, ref g2) => {
                self.obligations.extend([g1, g2]
                    .iter()
                    .map(|&goal| {
                        Obligation {
                            environment: environment.clone(),
                            goal: goal.clone(),
                        }
                    }));
                Ok(())
            }
            GoalKind::Or(ref g1, ref g2) => {
                for &goal in &[g1, g2] {
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
            GoalKind::Exists(ref quant) => {
                assert!(quant.num_binders > 0);
                let mut subst = None;
                for _ in 0..quant.num_binders {
                    let var = self.infer.new_variable(environment.universe_index()).to_leaf();
                    subst = Some(Subst::new(subst.as_ref(), var));
                }
                let subst = subst.unwrap(); // always at least 1 binder
                let new_goal = subst.apply(quant.skip_binders());
                self.obligations.push(Obligation {
                    environment: environment.clone(),
                    goal: new_goal,
                });
                Ok(())
            }
            GoalKind::ForAll(ref quant) => {
                assert!(quant.num_binders > 0);
                let mut new_environment = environment;
                let mut subst = None;
                for _ in 0..quant.num_binders {
                    new_environment = Arc::new(Environment::new(Some(new_environment), vec![]));
                    let depth = new_environment.depth();
                    subst = Some(Subst::new(subst.as_ref(), leaf!(apply (skol depth))));
                }
                let subst = subst.unwrap(); // always at least 1 binder
                let new_goal = subst.apply(quant.skip_binders());
                self.obligations.push(Obligation {
                    environment: new_environment,
                    goal: new_goal,
                });
                Ok(())
            }
            GoalKind::Implication(ref quant, ref goal) => unimplemented!(),
        }
    }
}
