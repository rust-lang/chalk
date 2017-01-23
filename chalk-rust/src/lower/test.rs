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
        assert_eq!(format!("{:?}", goal), "ForAll<type> { Exists<type> { ForAll<type> { ?0: Foo<?1, ?2> } } }");
    });
}

#[test]
fn atc_accounting() {
    let program = Arc::new(parse_and_lower("
            struct Vec<T> { }

            trait Iterable {
                type Iter<'a>;
            }

            impl<T> Iterable for Vec<T> {
                type Iter<'a> = Iter<'a, T>;
            }

            struct Iter<'a, T> { }
    ").unwrap());
    set_current_program(&program, || {
        let impl_text = format!("{:#?}", &program.impl_data.values().next().unwrap());
        println!("{}", impl_text);
        assert_eq!(&impl_text[..], r#"ImplData {
    crate_name: "crate",
    parameter_kinds: [
        "T"
    ],
    trait_ref: Vec<?0> as Iterable,
    where_clauses: [],
    assoc_ty_values: [
        AssocTyValue {
            associated_ty_id: (Iterable::Iter),
            value: Binders {
                binders: [
                    "\'a"
                ],
                value: AssocTyValueData {
                    ty: Iter<'?0, ?1>,
                    where_clauses: []
                }
            }
        }
    ]
}"#);
        let goal = parse_and_lower_goal(&program, "forall<X> { forall<'a> { forall<Y> { \
                                                   X: Iterable<Iter<'a> = Y> } } }")
            .unwrap();
        let goal_text = format!("{:?}", goal);
        println!("{}", goal_text);
        assert_eq!(goal_text, "ForAll<type> { ForAll<lifetime> { ForAll<type> { <?2 as Iterable>::Iter<'?1> ==> ?0 } } }");
    });
}
