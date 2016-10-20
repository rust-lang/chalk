use chalk_parse::ast;
use formula::leaf::Leaf;
use formula::goal::*;

use super::environment::Environment;
use super::lower_clause::LowerClause;
use super::lower_leaf::LowerLeaf;
use super::LowerResult;

pub trait LowerGoal<L> {
    fn lower_goal(&self, env: &mut Environment) -> LowerResult<Goal<L>>;
}

impl LowerGoal<Leaf> for ast::Application {
    fn lower_goal(&self, env: &mut Environment) -> LowerResult<Goal<Leaf>> {
        // collect the wildcards and bring them into scope
        let wildcards = self.count_wildcards();
        env.push_wildcards(wildcards);
        let leaf = self.lower_leaf(env)?;
        let goal = Goal::new(GoalData { kind: GoalKind::Leaf(leaf) });
        let goal = goal.in_exists(wildcards);
        env.pop_wildcards(wildcards);
        Ok(goal)
    }
}

impl LowerGoal<Leaf> for ast::Fact {
    fn lower_goal(&self, env: &mut Environment) -> LowerResult<Goal<Leaf>> {
        match *self.data {
            ast::FactData::And(ref f1, ref f2) => {
                let c1 = f1.lower_goal(env)?;
                let c2 = f2.lower_goal(env)?;
                Ok(Goal::new(GoalData { kind: GoalKind::And(vec![c1, c2]) }))
            }

            ast::FactData::Or(ref f1, ref f2) => {
                let c1 = f1.lower_goal(env)?;
                let c2 = f2.lower_goal(env)?;
                Ok(Goal::new(GoalData { kind: GoalKind::Or(vec![c1, c2]) }))
            }

            ast::FactData::Implication(ref f1, ref f2) => {
                let condition = f1.lower_clause(env)?;
                let consequence = f2.lower_goal(env)?;
                Ok(Goal::new(GoalData {
                    kind: GoalKind::Implication(condition, consequence),
                }))
            }

            ast::FactData::ForAll(v, ref f1) => {
                env.push_bound_name(v);
                let c = f1.lower_goal(env)?;
                env.pop_bound_name();
                Ok(c.in_foralls(1))
            }

            ast::FactData::Exists(v, ref f1) => {
                env.push_bound_name(v);
                let c = f1.lower_goal(env)?;
                env.pop_bound_name();
                Ok(c.in_exists(1))
            }

            ast::FactData::Apply(ref appl) => {
                appl.lower_goal(env)
            }
        }
    }
}
