use ast;
use lalrpop_intern::{intern, InternedString};
use std::fmt::Write;

impl ast::Application {
    pub fn intern_operator_name(&self) -> InternedString {
        let mut operator_name = String::new();
        let mut values: usize = 0;
        for bit in &self.bits {
            match bit.kind {
                ast::BitKind::Value(_) => values += 1,
                ast::BitKind::Operator(ast::Operator::Colon(name)) => {
                    write!(&mut operator_name, "{}", name).unwrap()
                }
                ast::BitKind::Operator(ast::Operator::Parens(name)) => {
                    write!(&mut operator_name, "{}()", name).unwrap()
                }
                ast::BitKind::Operator(ast::Operator::Symbols(name)) => {
                    write!(&mut operator_name, "{}", name).unwrap()
                }
            }
        }
        write!(&mut operator_name, "/{}", values).unwrap();
        intern(&operator_name)
    }
}
