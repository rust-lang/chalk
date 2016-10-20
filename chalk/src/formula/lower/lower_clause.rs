use chalk_parse::ast;
use formula::clause::*;
use formula::leaf::Leaf;

use super::LowerResult;
use super::lower_leaf::LowerLeaf;
use super::lower_goal::LowerGoal;
use super::environment::Environment;
use super::Error;
use super::ErrorKind;

pub trait LowerClause<L> {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<L>>;
}

impl LowerClause<Leaf> for ast::Item {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
        // bring all free variables into scope but ignore wildcards:
        let mut count = 0;
        self.for_each_free_variable(&mut |_span, v| {
            count += 1;
            env.push_bound_name(v);
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

        for _ in 0 .. count {
            env.pop_bound_name();
        }

        Ok(clause.in_foralls(count))
    }
}

impl LowerClause<Leaf> for ast::Application {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
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
        Ok(Clause::new(ClauseData { kind: ClauseKind::Implication(condition, consequence) }))
    }
}

impl LowerClause<Leaf> for ast::Fact {
    fn lower_clause(&self, env: &mut Environment) -> LowerResult<Clause<Leaf>> {
        panic!()
    }
}
