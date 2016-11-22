use chalk_parse::ast;
use formula::leaf::*;

use super::error::{LowerResult, Error, ErrorKind};
use super::environment::LowerEnvironment;

pub trait LowerLeaf<L> {
    fn lower_leaf(&self, env: &mut LowerEnvironment) -> LowerResult<L>;
}

impl LowerLeaf<Leaf> for ast::Application {
    fn lower_leaf(&self, env: &mut LowerEnvironment) -> LowerResult<Leaf> {
        let any_opers = self.bits.iter().any(|bit| match bit.kind {
            ast::BitKind::Operator(_) => true,
            ast::BitKind::Value(_) => false,
        });

        if any_opers {
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
                kind: LeafKind::Application(Application {
                    constant: Constant::Program(operator_name),
                    args: args,
                }),
            }))
        } else {
            // if no operators, must just be a value
            assert!(self.bits.len() == 1);
            let value = match self.bits[0].kind {
                ast::BitKind::Value(ref v) => v,
                ast::BitKind::Operator(_) => panic!("no operators"),
            };
            value.lower_leaf(env)
        }
    }
}

impl LowerLeaf<Leaf> for ast::Value {
    fn lower_leaf(&self, env: &mut LowerEnvironment) -> LowerResult<Leaf> {
        match self.kind {
            ast::ValueKind::Atom(atom) => {
                Ok(Leaf::new(LeafData {
                    kind: LeafKind::Application(Application {
                        constant: Constant::Program(atom.id),
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
                            path: env.path(),
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
