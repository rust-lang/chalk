#![feature(conservative_impl_trait)]

extern crate lalrpop_intern;
extern crate lalrpop_util;

pub mod ast;
mod parser;

use lalrpop_util::ParseError;

pub fn parse_program(input: &str)
                     -> Result<ast::Program,
                               ParseError<usize,(usize, &str),()>> {
    parser::parse_Program(input)
}

pub fn parse_goal(input: &str)
                  -> Result<ast::Fact,
                            ParseError<usize,(usize, &str),()>> {
    parser::parse_Goal(input)
}

#[test]
fn test_program() {
    let ast = parse_program("
Env |- X : Type :-
    !,
    Env |- E : (struct: S),
    struct: S has_field: F with_type: Type,
    foo(Bar, Baz).
").unwrap();
    println!("{:#?}", ast);
}
