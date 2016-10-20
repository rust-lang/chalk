use chalk_parse::ast;
use formula::leaf::Leaf;
use formula::goal::Goal;

use super::environment::Environment;
use super::LowerResult;

pub trait LowerGoal<L> {
    fn lower_goal(&self, env: &mut Environment) -> LowerResult<Goal<L>>;
}

impl LowerGoal<Leaf> for ast::Fact {
    fn lower_goal(&self, env: &mut Environment) -> LowerResult<Goal<Leaf>> {
        panic!()
    }
}
