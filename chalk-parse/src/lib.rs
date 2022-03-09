#![recursion_limit = "1024"]
#![allow(unused_parens)]

#[macro_use]
extern crate lalrpop_util;

pub mod ast;
#[rustfmt::skip]
lalrpop_mod!(pub parser);

use lalrpop_util::ParseError;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse_program(text: &str) -> Result<ast::Program> {
    parser::ProgramParser::new()
        .parse(text)
        .map_err(|e| format!("parse error: {}", e).into())
}

pub fn parse_ty(text: &str) -> Result<ast::Ty> {
    parser::TyParser::new()
        .parse(text)
        .map_err(|e| format!("error parsing `{}`: {}", text, e).into())
}

pub fn parse_goal(text: &str) -> Result<Box<ast::Goal>> {
    parser::GoalParser::new().parse(text).map_err(|e| {
        let mut output = format!("parse error: {}", &e);
        if let Some(s) = match e {
            ParseError::InvalidToken { location } => {
                Some(position_string(text, location, location + 1))
            }
            ParseError::UnrecognizedToken {
                token: (start, _, end),
                ..
            } => Some(position_string(text, start, end)),
            ParseError::ExtraToken {
                token: (start, _, end),
                ..
            } => Some(position_string(text, start, end)),
            _ => None,
        } {
            output.push('\n');
            output += &s;
        }
        output.into()
    })
}

fn position_string(text: &str, start: usize, end: usize) -> String {
    let text = text.replace('\n', " ").replace('\r', " ");
    let mut output = format!("position: `{}`", text);
    output += &" ".repeat(11 + start);
    output += &"^".repeat(end - start);
    output.push('\n');
    output
}
