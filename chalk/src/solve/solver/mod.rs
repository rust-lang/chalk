#![allow(unused_variables)]

use infer::*;
use formula::*;
use solve::*;
use subst::Subst;
use std::sync::Arc;

pub struct Solver {
    infer: InferenceTable,
    root_goal: Goal<Application>,
    solutions: Vec<String>,
    obligations: Vec<Obligation>,
}

impl Solver {
    pub fn solve(root_environment: Arc<Environment>, root_goal: Goal<Application>) -> Vec<String> {
        // Peel off any our "exists" goals and instantiate them with inference variables.
        let mut solver = Solver::new(&root_environment, &root_goal);
        solver.run();
        solver.solutions
    }

    fn new(root_environment: &Arc<Environment>, root_goal: &Goal<Application>) -> Self {
        let mut infer = InferenceTable::new();
        let root_goal = infer.peel_goal(root_environment, root_goal);
        Solver {
            infer: infer,
            root_goal: root_goal.clone(),
            solutions: vec![],
            obligations: vec![Obligation::new(root_environment.clone(), root_goal)]
        }
    }

    fn new_variable(&mut self, ui: UniverseIndex) -> InferenceVariable {
        self.infer.new_variable(ui)
    }

    fn canonicalize(&mut self, goal: &Goal<Application>) -> Goal<Application> {
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
        self.solutions.push(format!("{:?}", goal));
    }

    fn solve_obligation(&mut self, obligation: Obligation) -> Result<(), ()> {
        println!("solve_obligation: {:?}", obligation);
        let Obligation { environment, goal } = obligation;
        match goal.kind {
            GoalKind::True => Ok(()),
            GoalKind::Leaf(ref application) => {
                for clause in environment.clauses_relevant_to(application) {
                    self.probe(|this| {
                        let ClauseImplication { condition, consequence } =
                            this.infer.instantiate_existential(&environment, clause);

                        assert_eq!(application.constant_and_arity(),
                                   consequence.constant_and_arity());
                        for (leaf1, leaf2) in application.args.iter().zip(&consequence.args) {
                            if let Err(e) = this.infer.unify(leaf1, leaf2) {
                                println!("Unification error: {:?}", e);
                                return;
                            }
                        }

                        if let Some(goal) = condition {
                            this.obligations.push(Obligation {
                                environment: environment.clone(),
                                goal: goal,
                            });
                        }

                        this.run();
                    });
                }
                Err(())
            }
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
                let new_goal = self.infer.instantiate_existential(&environment, quant);
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
            GoalKind::Implication(ref clauses, ref goal) => {
                let new_environment = Arc::new(Environment::new(Some(environment),
                                                                clauses.clone()));
                self.obligations.push(Obligation {
                    environment: new_environment,
                    goal: goal.clone()
                });
                Ok(())
            }
        }
    }
}

impl InferenceTable {
    fn instantiate_existential<F>(&mut self,
                                  environment: &Environment,
                                  quant: &Quantification<F>)
                                  -> F
        where F: Fold + Clone
    {
        let mut subst = None;
        for _ in 0..quant.num_binders {
            let var = self.new_variable(environment.universe_index()).to_leaf();
            subst = Some(Subst::new(subst.as_ref(), var));
        }
        subst.map(|subst| subst.apply(quant.skip_binders()))
             .unwrap_or(quant.skip_binders().clone())
    }

    fn peel_goal(&mut self, root_environment: &Arc<Environment>, goal: &Goal<Application>)
                 -> Goal<Application>
    {
        let mut goal = goal.clone();

        // If the goal is `(exists X -> ...)`, then we instantiate `X`
        // with an inference variable and set `...` as our new "root
        // goal". This way, when we find solutions, we will print out
        // the value of `X` that made it true, and not just `exists X
        // -> ...`.
        loop {
            match goal.clone().kind {
                GoalKind::Exists(ref quant) => {
                    let formula = self.instantiate_existential(root_environment, quant);
                    goal = formula;
                }
                _ => break,
            }
        }
        goal
    }

}

#[cfg(test)]
mod test;
