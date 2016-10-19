use chalk_parse::ast;
use formula::leaf::*;

use super::LowerResult;
use super::environment::Environment;
use super::Error;
use super::ErrorKind;

pub trait LowerLeaf {
    fn lower_leaf(&self, env: &mut Environment) -> LowerResult<Leaf>;
}

impl LowerLeaf for ast::Application {
    fn lower_leaf(&self, env: &mut Environment) -> LowerResult<Leaf> {
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
    fn lower_leaf(&self, env: &mut Environment) -> LowerResult<Leaf> {
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
                let depth = env.claim_wildcard();
                Ok(Leaf::new(LeafData {
                    kind: LeafKind::BoundVariable(BoundVariable {
                        depth: depth
                    })
                }))
            }
        }
    }
}
