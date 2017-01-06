#![cfg(test)]

use chalk_rust_parse;
use errors::*;
use ir::*;
use std::sync::Arc;
use super::{LowerProgram, LowerGoal};

fn parse_and_lower(text: &str) -> Result<Program> {
    chalk_rust_parse::parse_program(text)?.lower()
}

fn parse_and_lower_goal(program: &Program, text: &str) -> Result<Box<Goal>> {
    chalk_rust_parse::parse_goal(text)?.lower(program)
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

#[test]
fn type_parameter_bound() {
    parse_and_lower("struct Foo { } trait Bar { } trait Eq { } impl<X> Bar for X where X: Eq { }")
        .unwrap();
}

#[test]
fn assoc_tys() {
    parse_and_lower("
    struct String { }
    struct Char { }

    trait Iterator { type Item; }
    impl Iterator for String { type Item = Char; }

    trait Foo { }
    impl<X> Foo for <X as Iterator>::Item where X: Iterator { }
    ")
        .unwrap();
}

#[test]
fn goal_quantifiers() {
    let program = Arc::new(parse_and_lower("trait Foo { }").unwrap());
    let goal = parse_and_lower_goal(&program, "forall<X> {exists<Y> {forall<Z> {Z: Foo<Y, X>}}}")
        .unwrap();
    set_current_program(&program, || {
        assert_eq!(format!("{:?}", goal), "ForAll(1, Exists(1, ForAll(1, Leaf(?2: Foo<?1, ?0>))))");
    });
}
