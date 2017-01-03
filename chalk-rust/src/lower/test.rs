#![cfg(test)]

use chalk_rust_parse;
use errors::*;
use ir;
use super::LowerProgram;

fn parse_and_lower(text: &str) -> Result<ir::Program> {
    chalk_rust_parse::parse_program(text)?.lower()
}

#[test]
fn lower() {
    parse_and_lower("struct Foo { } trait Bar { } impl Bar for Foo { }").unwrap();
}

#[test]
fn not_trait() {
    parse_and_lower("struct Foo { } trait Bar { } impl Foo for Bar { }").unwrap_err();
}

#[test]
fn invalid_name() {
    parse_and_lower("struct Foo { } trait Bar { } impl Bar for X { }").unwrap_err();
}

#[test]
fn type_parameter() {
    parse_and_lower("struct Foo { } trait Bar { } impl<X> Bar for X { }").unwrap();
}
