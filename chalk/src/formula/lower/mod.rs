use chalk_parse::ast::{self, Span};
use super::leaf::*;
use super::goal::Goal;
use super::clause::Clause;

pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

pub enum ErrorKind {
    UnknownVariable(ast::Variable),
}

pub type LowerResult<L> = Result<L, Error>;

pub struct Environment {
    bound_names: Vec<ast::Variable>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { bound_names: vec![] }
    }

    pub fn push(&mut self, v: ast::Variable) {
        self.bound_names.push(v);
    }

    pub fn pop(&mut self) {
        self.bound_names.pop();
    }

    pub fn lookup(&self, name: ast::Variable) -> Option<usize> {
        self.bound_names
            .iter()
            .rev()
            .position(|&x| x == name)
    }
}

pub trait LowerLeaf {
    fn lower_leaf(&self, env: &Environment) -> LowerResult<Leaf>;
}

pub trait LowerClause<L> {
    fn lower_clause(&self) -> LowerResult<Clause<L>>;
}

pub trait LowerGoal<L> {
    fn lower_clause(&self) -> LowerResult<Goal<L>>;
}

impl LowerLeaf for ast::Application {
    fn lower_leaf(&self, env: &Environment) -> LowerResult<Leaf> {
        let operator_name = self.intern_operator_name();
        let args: Vec<Leaf> = try!(self.bits
            .iter()
            .filter_map(|bit| match bit.kind {
                ast::BitKind::Value(ref v) => Some(v),
                ast::BitKind::Operator(_) => None,
            })
            .map(|v| v.lower_leaf(env))
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
    fn lower_leaf(&self, env: &Environment) -> LowerResult<Leaf> {
        match self.kind {
            ast::ValueKind::Atom(atom) => {
                Ok(Leaf::new(LeafData {
                    kind: LeafKind::Constant(Constant {
                        operator: atom.id,
                        args: vec![],
                    }),
                }))
            }

            ast::ValueKind::Variable(name) => {
                match env.lookup(name) {
                    Some(depth) => {
                        Ok(Leaf::new(LeafData {
                            kind: LeafKind::BoundVariable(BoundVariable {
                                depth: depth
                            })
                        }))
                    }
                    None => {
                        Err(Error {
                            span: self.span,
                            kind: ErrorKind::UnknownVariable(name),
                        })
                    }
                }
            }

            ast::ValueKind::Application(ref appl) => {
                appl.lower_leaf(env)
            }

            ast::ValueKind::Wildcard => {
                Ok(Leaf::new(LeafData {
                    kind: LeafKind::Wildcard
                }))
            }
        }
    }
}
