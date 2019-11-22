#![recursion_limit = "1024"]

#[macro_use]
extern crate lalrpop_util;

pub mod ast;
#[rustfmt::skip]
lalrpop_mod!(pub parser);

use lalrpop_util::ParseError;
use std::fmt::Write;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_program(text: &str) -> Result<ast::Program> {
    match parser::ProgramParser::new().parse(text) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("parse error: {:?}", e))?,
    }
}

pub fn parse_ty(text: &str) -> Result<ast::Ty> {
    match parser::TyParser::new().parse(text) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("error parsing `{}`: {:?}", text, e))?,
    }
}

pub fn parse_goal(text: &str) -> Result<Box<ast::Goal>> {
    match parser::GoalParser::new().parse(text) {
        Ok(v) => Ok(v),
        Err(e) => {
            let position_string = |start: usize, end: usize| {
                let mut output = String::new();
                let text = text.replace("\n", " ").replace("\r", " ");
                writeln!(output, "position: `{}`", text).expect("str-write cannot fail");
                output.push_str(&" ".repeat(11 + start));
                output.push_str(&"^".repeat(end - start));
                output.push_str("\n");
                output
            };
            match e {
                ParseError::InvalidToken { location } => Err(format!(
                    "parse error: {:?}\n{}",
                    e,
                    position_string(location, location + 1)
                ))?,
                ParseError::UnrecognizedToken {
                    token: (start, _, end),
                    ..
                } => Err(format!(
                    "parse error: {:?}\n{}",
                    e,
                    position_string(start, end)
                ))?,
                ParseError::ExtraToken {
                    token: (start, _, end),
                    ..
                } => Err(format!(
                    "parse error: {:?}\n{}",
                    e,
                    position_string(start, end)
                ))?,
                _ => Err(format!("parse error: {:?}", e))?,
            }
        }
    }
}
