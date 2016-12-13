#![allow(unused_variables)]

use infer::*;
use formula::*;
use solve::*;
use subst::Subst;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct Solver {
    infer: InferenceTable,
    root_goal: Goal<Application>,
    solutions: Vec<String>,
    obligations: Vec<Obligation>,
    choice_points: Vec<ChoicePoint>,
}

struct ChoicePoint {
    obligations: Vec<Obligation>,
    infer_snapshot: InferenceSnapshot,
    kind: ChoicePointKind,
}

enum ChoicePointKind {
    Clauses(ChoicePointClauses),
    Disjunction(ChoicePointDisjunction),
}

struct ChoicePointClauses {
    clauses: VecDeque<Clause<Application>>,
    environment: Arc<Environment>,
    application: Application,
    depth: usize,
}

struct ChoicePointDisjunction {
    goals: VecDeque<Obligation>,
}

enum ProveError {
    NotProvable,
    Overflow,
}

// Indicates that there are no more choicepoints
struct UnrollError;

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
            obligations: vec![Obligation::new(root_environment.clone(), root_goal, 0)],
            choice_points: vec![],
        }
    }

    fn canonicalize(&mut self, goal: &Goal<Application>) -> Goal<Application> {
        // FIXME -- this meant to replace unbound variables like ?F
        // with a `_`, but that is not a variant of leaf (and should
        // not be). Should be able to fix this without extending leaf
        // but might need to generalize folder trait.
        self.infer.normalize_deep(goal)
    }

    fn run(&mut self) {
        loop {
            match self.find_next_solution() {
                Ok(solution) => {
                    self.solutions.push(solution);
                }
                Err(ProveError::NotProvable) => {
                }
                Err(ProveError::Overflow) => {
                    if !self.solutions.iter().any(|s| s == "<<overflow>>") {
                        self.solutions.push("<<overflow>>".to_string());
                    }
                }
            }

            match self.unroll() {
                Ok(()) => { }
                Err(UnrollError) => {
                    return;
                }
            }
        }
    }

    fn find_next_solution(&mut self) -> Result<String, ProveError> {
        while let Some(obligation) = self.obligations.pop() {
            self.solve_obligation(obligation)?;
        }

        let goal = self.root_goal.clone();
        let goal = self.canonicalize(&goal);
        Ok(format!("{:?}", goal))
    }

    fn unroll(&mut self) -> Result<(), UnrollError> {
        if let Some(top_choice_point) = self.choice_points.pop() {
            let ChoicePoint { obligations, infer_snapshot, kind } = top_choice_point;
            self.obligations = obligations;
            self.infer.rollback_to(infer_snapshot);
            match kind {
                ChoicePointKind::Clauses(clauses) => {
                    self.start_next_clause(clauses)
                }
                ChoicePointKind::Disjunction(disjunction) => {
                    self.start_next_disjunction(disjunction)
                }
            }
        } else {
            Err(UnrollError)
        }
    }

    fn start_next_clause(&mut self, clauses: ChoicePointClauses) -> Result<(), UnrollError> {
        let ChoicePointClauses { mut clauses, application, environment, depth } = clauses;

        'next_clause: while let Some(clause) = clauses.pop_front() {
            let snapshot = self.infer.snapshot();

            let ClauseImplication { condition, consequence } =
                self.infer.instantiate_existential(&environment, &clause);

            assert_eq!(application.constant_and_arity(),
                       consequence.constant_and_arity());
            for (leaf1, leaf2) in application.args.iter().zip(&consequence.args) {
                if let Err(e) = self.infer.unify(leaf1, leaf2) {
                    self.infer.rollback_to(snapshot);
                    continue 'next_clause;
                }
            }

            self.choice_points.push(ChoicePoint {
                obligations: self.obligations.clone(),
                infer_snapshot: snapshot,
                kind: ChoicePointKind::Clauses(ChoicePointClauses {
                    environment: environment.clone(),
                    clauses: clauses,
                    application: application,
                    depth: depth,
                })
            });

            if let Some(goal) = condition {
                self.obligations.push(Obligation {
                    environment: environment.clone(),
                    goal: goal,
                    depth: depth,
                });
            }

            return Ok(());
        }

        self.unroll()
    }

    fn start_next_disjunction(&mut self, disjunction: ChoicePointDisjunction) -> Result<(), UnrollError> {
        let ChoicePointDisjunction { mut goals } = disjunction;

        while let Some(goal) = goals.pop_front() {
            let snapshot = self.infer.snapshot();

            self.choice_points.push(ChoicePoint {
                obligations: self.obligations.clone(),
                infer_snapshot: snapshot,
                kind: ChoicePointKind::Disjunction(ChoicePointDisjunction { goals: goals }),
            });

            self.obligations.push(goal);

            return Ok(());
        }

        self.unroll()
    }

    fn solve_obligation(&mut self, obligation: Obligation) -> Result<(), ProveError> {
        debug!("solve_obligation(obligation={:#?})", obligation);
        debug!("solve_obligation: goal={:?}", self.canonicalize(&obligation.goal));
        let Obligation { environment, goal, depth } = obligation;
        if depth > 10 {
            return Err(ProveError::Overflow);
        }
        match goal.kind {
            GoalKind::True => Ok(()),
            GoalKind::Leaf(ref application) => {
                let clauses: VecDeque<_> = environment.clauses_relevant_to(application).cloned().collect();
                let clauses = ChoicePointClauses {
                    clauses: clauses,
                    environment: environment,
                    application: application.clone(),
                    depth: depth + 1,
                };

                match self.start_next_clause(clauses) {
                    Ok(()) => Ok(()),
                    Err(UnrollError) => Err(ProveError::NotProvable),
                }
            }
            GoalKind::And(ref g1, ref g2) => {
                // NB: Important that we consider g1 first
                self.obligations.extend([g2, g1]
                    .iter()
                    .map(|&goal| {
                        Obligation {
                            environment: environment.clone(),
                            goal: goal.clone(),
                            depth: depth,
                        }
                    }));
                Ok(())
            }
            GoalKind::Or(ref g1, ref g2) => {
                let mut deque = VecDeque::new();
                deque.push_back(Obligation {
                    environment: environment.clone(),
                    goal: g1.clone(),
                    depth: depth + 1,
                });
                deque.push_back(Obligation {
                    environment: environment.clone(),
                    goal: g2.clone(),
                    depth: depth + 1,
                });
                let disjunction = ChoicePointDisjunction {
                    goals: deque,
                };
                match self.start_next_disjunction(disjunction) {
                    Ok(()) => Ok(()),
                    Err(UnrollError) => Err(ProveError::NotProvable),
                }
            }
            GoalKind::Exists(ref quant) => {
                let new_goal = self.infer.instantiate_existential(&environment, quant);
                self.obligations.push(Obligation {
                    environment: environment.clone(),
                    goal: new_goal,
                    depth: depth,
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
                    depth: depth,
                });
                Ok(())
            }
            GoalKind::Implication(ref clauses, ref goal) => {
                let new_environment = Arc::new(Environment::new(Some(environment),
                                                                clauses.clone()));
                self.obligations.push(Obligation {
                    environment: new_environment,
                    goal: goal.clone(),
                    depth: depth,
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
