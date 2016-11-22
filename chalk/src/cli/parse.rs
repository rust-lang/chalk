use chalk_parse::{self, ast};
use lalrpop_util::ParseError;
use std::error::Error;
use std::fmt;

pub fn parse_program(path: &str, input: &str) -> Result<ast::Program, Box<Error>> {
    match chalk_parse::parse_program(input) {
        Ok(p) => Ok(p),
        Err(err) => {
            let (location, text) = match err {
                ParseError::InvalidToken { location: l } => (l, "invalid token"),
                ParseError::UnrecognizedToken { token: Some((l, _, _)), .. } => (l, "unrecognized token"),
                ParseError::UnrecognizedToken { token: None, .. } => (input.len(), "unexpected EOF"),
                ParseError::ExtraToken { token: (l, _, _) } => (l, "extra token"),
                ParseError::User { .. } => panic!("no user-defined errors"),
            };

            let line_num = input[..location].lines().count();
            let line_start = input[..location].rfind('\n').unwrap_or(0);
            let col_num = input[line_start..location].chars().count();
            Err(HumanParseError {
                path: path.to_string(),
                line_num: line_num,
                col_num: col_num,
                description: text
            })?;
            unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct HumanParseError {
    path: String,
    line_num: usize,
    col_num: usize,
    description: &'static str,
}

impl Error for HumanParseError {
    fn description(&self) -> &str {
        self.description
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for HumanParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "parse error at {}:{}:{}: {}",
               self.path, self.line_num, self.col_num, self.description)
    }
}
