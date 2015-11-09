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

#[test]
fn test_program() {
    let ast = parse_program("
Env |- [E '.' F] : Type :-
    !,
    Env |- E : [struct: S],
    struct: S has_field: F with_type: Type.
").unwrap();
    println!("{:#?}", ast);
}
