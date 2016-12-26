use chalk_parse::ast;
use formula::*;
use std::collections::HashSet;

use super::error::{LowerResult, Error, ErrorKind};
use super::lower_application::LowerApplication;
use super::lower_goal::LowerGoal;
use super::environment::LowerEnvironment;

pub trait LowerClause<L> {
    fn lower_clause(&self, env: &mut LowerEnvironment) -> LowerResult<Vec<Clause<L>>>;
}

impl LowerClause<Application> for ast::Item {
    fn lower_clause(&self, env: &mut LowerEnvironment) -> LowerResult<Vec<Clause<Application>>> {
        debug!("Item lower_clause");

        // bring all free variables into scope but ignore wildcards:
        let mut count = 0;
        let mut set = HashSet::new();
        self.for_each_free_variable(&mut |_span, v| {
            if set.insert(v.id) {
                count += 1;
                env.push_bound_name(v);
            }
        });

        // this is because we want to transform something like
        //
        //     Foo(A, _, B) :- Bar(A, _, B, C).
        //
        // to
        //
        //     forall(A, B, C -> exists(WC1 -> Bar(A, WC1, B, C)) => forall(WC2 -> Foo(A, WC2, B)))
        //
        // which then gets normalized to
        //
        //     forall(A, B, C, WC2 -> exists(WC1 -> Bar(A, WC1, B, C)) => Foo(A, WC2, B))
        //
        // note that A, B, and C all appear at the top-level, but WC1
        // and WC2 are distinct variables. This matters if you have nested
        // forall binders, like this:
        //
        //      Foo(A, B) :- forall(X -> Bar(A, B, X, _)).
        //
        // In particular, we want to translate this so that the `_` can be bound to `X`.
        // See the test `lower_nested_wildcard`; lowering the above yields
        //
        //      forall(A, B -> forall(C -> exists(D -> Bar(A, B, C, D))) => Foo(A, B))
        //      //                                ^ the wildcard

        let clauses = match *self {
            ast::Item::Fact(ref appl) => appl.lower_clause(env),
            ast::Item::Rule(ref rule) => rule.lower_clause(env),
        }?;

        for _ in 0..count {
            env.pop_bound_name();
        }

        Ok(clauses.into_iter()
            .map(|clause| in_foralls(clause, count))
            .collect())
    }
}

impl LowerClause<Application> for ast::Application {
    fn lower_clause(&self, env: &mut LowerEnvironment) -> LowerResult<Vec<Clause<Application>>> {
        debug!("Application lower_clause");

        // collect the wildcards and bring them into scope
        let wildcards = self.count_wildcards();
        env.push_wildcards(wildcards);
        let application = self.lower_application(env)?;
        let clause = clause!(leaf (expr application));
        let clause = in_foralls(clause, wildcards);
        env.pop_wildcards(wildcards);
        Ok(vec![clause])
    }
}

impl LowerClause<Application> for ast::Rule {
    fn lower_clause(&self, env: &mut LowerEnvironment) -> LowerResult<Vec<Clause<Application>>> {
        let consequences = self.consequence.lower_clause(env)?;
        let condition = self.condition.lower_goal(env)?;
        Ok(consequences.into_iter()
            .map(|consequence| consequence.flatten_implication(&condition))
            .collect())
    }
}

impl LowerClause<Application> for ast::Fact {
    fn lower_clause(&self, env: &mut LowerEnvironment) -> LowerResult<Vec<Clause<Application>>> {
        match *self.data {
            ast::FactData::Not(_) => {
                Err(Error {
                    path: env.path(),
                    span: self.span,
                    kind: ErrorKind::NotInClause,
                })
            }

            ast::FactData::And(ref f1, ref f2) => {
                let c1 = f1.lower_clause(env)?;
                let c2 = f2.lower_clause(env)?;
                Ok(append(c1, c2))
            }

            ast::FactData::Implication(ref f1, ref f2) => {
                let condition = f1.lower_goal(env)?;
                let consequences = f2.lower_clause(env)?;
                Ok(consequences.into_iter()
                    .map(|consequence| consequence.flatten_implication(&condition))
                    .collect())
            }

            ast::FactData::ForAll(v, ref f1) => {
                env.push_bound_name(v);
                let clauses = f1.lower_clause(env)?;
                env.pop_bound_name();
                Ok(clauses.into_iter()
                    .map(|clause| in_foralls(clause, 1))
                    .collect())
            }

            ast::FactData::Exists(..) => {
                Err(Error {
                    path: env.path(),
                    span: self.span,
                    kind: ErrorKind::ExistsInClause,
                })
            }

            ast::FactData::Apply(ref appl) => appl.lower_clause(env),

            ast::FactData::Or(..) => {
                Err(Error {
                    path: env.path(),
                    span: self.span,
                    kind: ErrorKind::OrInClause,
                })
            }

            ast::FactData::IfThenElse(..) => {
                Err(Error {
                    path: env.path(),
                    span: self.span,
                    kind: ErrorKind::IfThenElseInClause,
                })
            }
        }
    }
}

impl Clause<Application> {
    pub fn flatten_implication(&self, goal: &Goal<Application>) -> Clause<Application> {
        let goal = goal.fold_with(&mut OpenUp::new(self.num_binders));
        if let Some(ref goal2) = self.skip_binders().condition {
            clause!(forall(self.num_binders) implies (and (expr goal) (expr goal2))
                    => (expr self.skip_binders().consequence))
        } else {
            clause!(forall(self.num_binders) implies (expr goal)
                    => (expr self.skip_binders().consequence))
        }
    }
}

pub fn append<T>(mut v: Vec<T>, mut v2: Vec<T>) -> Vec<T> {
    v.append(&mut v2);
    v
}

fn in_foralls<L: Clone>(clause: Clause<L>, binders: usize) -> Clause<L> {
    Clause::new(Quantification::new(clause.num_binders + binders, clause.skip_binders().clone()))
}
