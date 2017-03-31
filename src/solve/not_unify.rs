use cast::Cast;
use errors::*;
use ir::*;
use solve::solver::Solver;
use solve::{Solution, Successful};
use std::mem;
use std::sync::Arc;

pub struct SolveNotUnify<'s> {
    solver: &'s mut Solver,
    binders: QueryBinders,
    environment: Arc<Environment>,
    goal: Not<Unify<Ty>>,
    state: State,
}

enum State {
    Unprovable,
    Ambiguous,
    IfGoalsMet(Vec<Query<InEnvironment<WhereClauseGoal>>>),
    NotUnifiable,
}
use self::State::*;

impl<'s> SolveNotUnify<'s> {
    pub fn new(solver: &'s mut Solver,
               env_goal: Query<InEnvironment<Not<Unify<Ty>>>>)
               -> Self {
        let Query { value: InEnvironment { environment, goal }, binders } = env_goal;
        SolveNotUnify { solver, binders, environment, goal, state: Unprovable }
    }

    pub fn solve(mut self) -> Result<Solution<InEnvironment<Not<Unify<Ty>>>>> {
        let successful = self.solve_tys()?;
        Ok(Solution {
            successful: successful,
            refined_goal: Query {
                binders: self.binders,
                value: Constrained {
                    value: InEnvironment {
                        environment: self.environment,
                        goal: self.goal,
                    },
                    constraints: vec![]
                }
            }
        })
    }

    fn solve_tys(&mut self) -> Result<Successful> {
        let Not { predicate: Unify { a, b }, krate: _ } = self.goal.clone();
        self.tys(&a, &b);
        match mem::replace(&mut self.state, Unprovable) {
            Unprovable => {
                bail!("`{:?} != {:?}` is unprovable", a, b)
            }
            Ambiguous => {
                Ok(Successful::Maybe)
            }
            IfGoalsMet(goals) => {
                for goal in goals {
                    match self.solver.solve(goal) {
                        Err(_) => { }
                        Ok(Solution { successful: Successful::Maybe, .. }) => {
                        }
                        Ok(Solution { successful: Successful::Yes, .. }) => {
                            return Ok(Successful::Yes);
                        }
                    }
                }
                Ok(Successful::Maybe)
            }
            NotUnifiable => {
                Ok(Successful::Yes)
            }
        }
    }

    fn unprovable(&mut self) {
    }

    fn ambiguous(&mut self) {
        debug!("ambiguous");
        match self.state {
            Unprovable => self.state = Ambiguous,
            _ => ()
        }
    }

    fn if_goal_met(&mut self, goal: Query<InEnvironment<WhereClauseGoal>>) {
        debug!("not_unifiable");
        match self.state {
            Unprovable | Ambiguous => self.state = IfGoalsMet(vec![goal]),
            IfGoalsMet(ref mut goals) => goals.push(goal),
            NotUnifiable => (),
        }
    }

    fn not_unifiable(&mut self) {
        debug!("not_unifiable");
        self.state = NotUnifiable;
    }

    fn tys<'a>(&mut self, a: &'a Ty, b: &'a Ty) {
        //             ^^                 ^^         ^^ FIXME rustc bug
        debug_heading!("tys(a={:?}\
                     ,\n    b={:?})", a, b);

        match (a, b) {
            (&Ty::Var(depth1), &Ty::Var(depth2)) if depth1 == depth2 => {
                self.unprovable();
            }

            (&Ty::Var(_), _) |
            (_, &Ty::Var(_)) => {
                self.ambiguous();
            }

            (&Ty::Apply(ref apply1), &Ty::Apply(ref apply2)) => {
                match (apply1.name, apply2.name) {
                    (TypeName::ItemId(n1), TypeName::ItemId(n2)) => {
                        if n1 != n2 {
                            self.not_unifiable();
                        } else {
                            for (param1, param2) in apply1.parameters.iter().zip(&apply2.parameters) {
                                self.parameters(param1, param2);
                            }
                        }
                    }

                    (_, TypeName::ForAll(_)) |
                    (_, TypeName::AssociatedType(_)) |
                    (TypeName::ForAll(_), _) |
                    (TypeName::AssociatedType(_), _) => {
                        // in both of these cases, we don't really know what
                        // these values are, so we can't say that they are disjoint
                        self.unprovable();
                    }
                }
            }

            (&Ty::ForAll(ref quantified_ty1), &Ty::ForAll(ref quantified_ty2)) => {
                self.forall_tys(quantified_ty1, quantified_ty2);
            }

            (&Ty::ForAll(ref quantified_ty), apply_ty @ &Ty::Apply(_)) |
            (apply_ty @ &Ty::Apply(_), &Ty::ForAll(ref quantified_ty)) => {
                self.forall_apply(quantified_ty, apply_ty);
            }

            (&Ty::Projection(ref proj1), &Ty::Projection(ref proj2)) => {
                self.projection_tys(proj1, proj2);
            }

            (ty @ &Ty::Apply(_), &Ty::Projection(ref proj)) |
            (ty @ &Ty::ForAll(_), &Ty::Projection(ref proj)) |
            (&Ty::Projection(ref proj), ty @ &Ty::Apply(_)) |
            (&Ty::Projection(ref proj), ty @ &Ty::ForAll(_)) => {
                self.projection_ty(proj, ty);
            }
        }
    }

    fn forall_tys(&mut self, ty1: &QuantifiedTy, ty2: &QuantifiedTy) {
        // for<'a...> T == for<'b...> U where 'a != 'b
        //
        // if:
        //
        // for<'a...> exists<'b...> T == U &&
        // for<'b...> exists<'a...> T == U

        debug!("forall_tys({:?}, {:?})", ty1, ty2);

        unimplemented!()
    }

    fn projection_tys(&mut self,
                      _proj1: &ProjectionTy,
                      _proj2: &ProjectionTy) {
        unimplemented!()
    }

    fn projection_ty(&mut self, proj: &ProjectionTy, ty: &Ty) {
        let goal = Query {
            binders: self.binders.clone(),
            value: InEnvironment::new(&self.environment,
                                      Not {
                                          predicate: Normalize {
                                              projection: proj.clone(),
                                              ty: ty.clone(),
                                          },
                                          krate: self.goal.krate,
                                      }.cast())
        };
        self.if_goal_met(goal)
    }

    fn forall_apply(&mut self, _ty1: &QuantifiedTy, _ty2: &Ty) {
        // should generate a forall != goal etc
        unimplemented!()
    }

    fn parameters(&mut self, p1: &Parameter, p2: &Parameter) {
        use ir::ParameterKind::*;
        match (p1, p2) {
            (&Ty(ref t1), &Ty(ref t2)) => self.tys(t1, t2),
            (&Lifetime(_), &Lifetime(_)) => self.unprovable(),
            (&Krate(_), &Krate(_)) => panic!("krate in type"),
            (&Ty(_), _) |
            (&Lifetime(_), _) |
            (&Krate(_), _) => {
                panic!("mismatched kinds")
            }
        }
    }
}
