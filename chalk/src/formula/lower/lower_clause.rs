use chalk_parse::ast;
use formula::*;
use std::collections::HashSet;

use super::lower_leaf::LowerLeaf;
use super::lower_goal::LowerGoal;
use super::environment::Environment;

pub trait LowerClause<L> {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Vec<Clause<L>>>;
}

impl LowerClause<Leaf> for ast::Item {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Vec<Clause<Leaf>>> {
        println!("Item lower_clause");

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
        // note that A, B, and C all appear at the top-level, but WC1
        // and WC2 are bound lower. This matters if you have nested
        // forall binders, like this:
        //
        //      Foo(A, B) :- forall(X -> Bar(A, B, X, _)).
        //
        // In particular, we want to translate this so that the `_` can be bound to `X`.

        let clauses = match *self {
            ast::Item::Fact(ref appl) => appl.lower_clause(env),
            ast::Item::Rule(ref rule) => rule.lower_clause(env),
        }?;

        for _ in 0..count {
            env.pop_bound_name();
        }

        Ok(clauses.into_iter()
            .map(|clause| clause.in_foralls(count))
            .collect())
    }
}

impl LowerClause<Leaf> for ast::Application {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Vec<Clause<Leaf>>> {
        println!("Application lower_clause");

        // collect the wildcards and bring them into scope
        let wildcards = self.count_wildcards();
        env.push_wildcards(wildcards);
        let leaf = self.lower_leaf(env)?;
        let clause = clause!(leaf (expr leaf));
        let clause = clause.in_foralls(wildcards);
        env.pop_wildcards(wildcards);
        Ok(vec![clause])
    }
}

impl LowerClause<Leaf> for ast::Rule {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Vec<Clause<Leaf>>> {
        let consequences = self.consequence.lower_clause(env)?;
        let condition = self.condition.lower_goal(env)?;
        Ok(consequences.into_iter()
            .map(|consequence| consequence.flatten_implication(&condition))
            .collect())
    }
}

impl LowerClause<Leaf> for ast::Fact {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Vec<Clause<Leaf>>> {
        match *self.data {
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
                    .map(|clause| clause.in_foralls(1))
                    .collect())
            }

            ast::FactData::Exists(..) => {
                Err(Error {
                    span: self.span,
                    kind: ErrorKind::ExistsInClause,
                })
            }

            ast::FactData::Apply(ref appl) => appl.lower_clause(env),

            ast::FactData::Or(..) => {
                Err(Error {
                    span: self.span,
                    kind: ErrorKind::OrInClause,
                })
            }
        }
    }
}

impl Clause<Leaf> {
    pub fn flatten_implication(&self, goal: &Goal<Leaf>) -> Clause<Leaf> {
        match self.kind {
            ClauseKind::Leaf(ref leaf) => clause!(implies (expr goal) => (expr leaf)),
            ClauseKind::Implication(ref goal2, ref leaf) => {
                clause!(implies (and (expr goal) (expr goal2)) => (expr leaf))
            }
            ClauseKind::ForAll(ref quant) => {
                let goal = goal.fold_with(&mut OpenUp::new(quant.num_binders));
                let formula = quant.formula.flatten_implication(&goal);
                clause!(forall(quant.num_binders) (expr formula))
            }
        }
    }
}

pub fn append<T>(mut v: Vec<T>, mut v2: Vec<T>) -> Vec<T> {
    v.append(&mut v2);
    v
}
