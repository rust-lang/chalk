use chalk_parse::ast;
use formula::leaf::*;

use super::LowerResult;
use super::environment::LowerEnvironment;
use super::Error;
use super::ErrorKind;
use super::lower_leaf::LowerLeaf;

pub trait LowerApplication {
    fn lower_application(&self, env: &mut LowerEnvironment) -> LowerResult<Application>;
}

impl LowerApplication for ast::Application {
    fn lower_application(&self, env: &mut LowerEnvironment) -> LowerResult<Application> {
        let any_opers = self.bits.iter().any(|bit| match bit.kind {
            ast::BitKind::Operator(_) => true,
            ast::BitKind::Value(_) => false,
        });

        if !any_opers {
            return Err(Error { span: self.span, kind: ErrorKind::NoOperator });
        }

        let operator_name = self.intern_operator_name();
        let args: Vec<Leaf> = try!(self.bits
                                   .iter()
                                   .filter_map(|bit| match bit.kind {
                                       ast::BitKind::Value(ref v) => Some(v),
                                       ast::BitKind::Operator(_) => None,
                                   })
                                   .map(|v| v.lower_leaf(env))
                                   .collect());
        Ok(Application {
            constant: Constant::Program(operator_name),
            args: args,
        })
    }
}
