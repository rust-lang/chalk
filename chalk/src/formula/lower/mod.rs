use chalk_parse::ast::{self, Span};
use super::leaf::*;
use super::goal::Goal;
use super::clause::Clause;

pub struct Error {
    pub span: Span,
}

pub enum ErrorKind {
    Misc,
}

pub type LowerResult<L> = Result<L, Error>;

pub trait LowerLeaf {
    fn lower_leaf(&self) -> LowerResult<Leaf>;
}

pub trait LowerClause<L> {
    fn lower_clause(&self) -> LowerResult<Clause<L>>;
}

pub trait LowerGoal<L> {
    fn lower_clause(&self) -> LowerResult<Goal<L>>;
}

impl LowerLeaf for ast::Application {
    fn lower_leaf(&self) -> LowerResult<Leaf> {
        let operator_name = self.intern_operator_name();
        let args: Vec<Leaf> = try!(self.bits.iter()
                                   .filter_map(|bit| match bit.kind {
                                       ast::BitKind::Value(ref v) => Some(v),
                                       ast::BitKind::Operator(_) => None,
                                   })
                                   .map(|v| v.lower_leaf())
                                   .collect());
        Ok(Leaf::new(LeafData {
            kind: LeafKind::Constant(Constant {
                operator: operator_name,
                args: args,
            }),
        }))
    }
}

impl LowerLeaf for ast::Value {
    fn lower_leaf(&self) -> LowerResult<Leaf> {
        unimplemented!()
    }
}
