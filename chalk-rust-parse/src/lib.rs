#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

extern crate lalrpop_intern;
extern crate lalrpop_util;

pub mod ast;
pub mod errors;
mod parser;

use errors::Result;

pub fn parse_program(text: &str) -> Result<ast::Program> {
    match parser::parse_Program(text) {
        Ok(v) => Ok(v),
        Err(e) => bail!("parse error: {:?}", e),
    }
}

pub fn parse_ty(text: &str) -> Result<ast::Ty> {
    match parser::parse_Ty(text) {
        Ok(v) => Ok(v),
        Err(e) => bail!("error parsing `{}`: {:?}", text, e),
    }
}
