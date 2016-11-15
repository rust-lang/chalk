use chalk_parse::ast;
use formula::*;
use std::collections::HashSet;

use super::lower_leaf::LowerLeaf;
use super::lower_goal::LowerGoal;
use super::environment::Environment;

pub trait LowerClause<L> {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<L>>;
}

impl LowerClause<Leaf> for ast::Item {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
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

        let clause = match *self {
            ast::Item::Fact(ref appl) => appl.lower_clause(env),
            ast::Item::Rule(ref rule) => rule.lower_clause(env),
        }?;

        for _ in 0..count {
            env.pop_bound_name();
        }

        Ok(clause.in_foralls(count))
    }
}

impl LowerClause<Leaf> for ast::Application {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
        println!("Application lower_clause");

        // collect the wildcards and bring them into scope
        let wildcards = self.count_wildcards();
        env.push_wildcards(wildcards);
        let leaf = self.lower_leaf(env)?;
        let clause = Clause::new(ClauseData { kind: ClauseKind::Leaf(leaf) });
        let clause = clause.in_foralls(wildcards);
        env.pop_wildcards(wildcards);
        Ok(clause)
    }
}

impl LowerClause<Leaf> for ast::Rule {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
        let consequence = self.consequence.lower_clause(env)?;
        let condition = self.condition.lower_goal(env)?;
        Ok(consequence.flatten_implication(&condition))
    }
}

impl LowerClause<Leaf> for ast::Fact {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
        match *self.data {
            ast::FactData::And(ref f1, ref f2) => {
                let c1 = f1.lower_clause(env)?;
                let c2 = f2.lower_clause(env)?;
                Ok(Clause::new(ClauseData { kind: ClauseKind::And(c1, c2) }))
            }

            ast::FactData::Implication(ref f1, ref f2) => {
                let condition = f1.lower_goal(env)?;
                let consequence = f2.lower_clause(env)?;
                Ok(consequence.flatten_implication(&condition))
            }

            ast::FactData::ForAll(v, ref f1) => {
                env.push_bound_name(v);
                let c = f1.lower_clause(env)?;
                env.pop_bound_name();
                Ok(c.in_foralls(1))
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
            ClauseKind::Leaf(ref leaf) => clause!((implies (expr goal) => (expr leaf))),
            ClauseKind::And(ref c1, ref c2) => {
                let c1_flat = c1.flatten_implication(goal);
                let c2_flat = c2.flatten_implication(goal);
                clause!((and (expr c1_flat) (expr c2_flat)))
            }
            ClauseKind::Implication(ref goal2, ref leaf) => {
                clause!((implies (and (expr goal) (expr goal2)) => (expr leaf)))
            }
            ClauseKind::ForAll(ref quant) => {
                let goal = goal.fold_with(&mut OpenUp::new(quant.num_binders));
                let formula = quant.formula.flatten_implication(&goal);
                clause!(forall(quant.num_binders) (expr formula))
            }
        }
    }
}
