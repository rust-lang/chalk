#![cfg(test)]

use chalk_parse;
use errors::*;
use ir::*;
use std::sync::Arc;
use super::{LowerProgram, LowerGoal};

fn parse_and_lower(text: &str) -> Result<Program> {
    chalk_parse::parse_program(text)?.lower()
}

fn parse_and_lower_goal(program: &Program, text: &str) -> Result<Box<Goal>> {
    chalk_parse::parse_goal(text)?.lower(program)
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
    let program = Arc::new(parse_and_lower("trait Foo<A, B> { }").unwrap());
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
        assert_eq!(&impl_text[..], r#"ImplDatum {
    krate_id: krate,
    binders: for<type> ImplDatumBound {
        trait_ref: Vec<?0> as Iterable,
        where_clauses: [],
        associated_ty_values: [
            AssociatedTyValue {
                associated_ty_id: (Iterable::Iter),
                value: for<lifetime> AssociatedTyValueBound {
                    ty: Iter<'?0, ?1>,
                    where_clauses: []
                }
            }
        ]
    }
}"#);
        let goal = parse_and_lower_goal(&program, "forall<X> { forall<'a> { forall<Y> { \
                                                   X: Iterable<Iter<'a> = Y> } } }")
            .unwrap();
        let goal_text = format!("{:?}", goal);
        println!("{}", goal_text);
        assert_eq!(goal_text, "ForAll<type> { ForAll<lifetime> { ForAll<type> { <?2 as Iterable>::Iter<'?1> ==> ?0 } } }");
    });
}

macro_rules! lowering_error {
    (program $program:tt error_msg { $expected:expr }) => {
        let program_text = stringify!($program);
        assert!(program_text.starts_with("{"));
        assert!(program_text.ends_with("}"));
        let error = parse_and_lower(&program_text[1..program_text.len()-1]).unwrap_err();
        let expected = Error::from($expected);
        assert_eq!(
            error.to_string(),
            expected.to_string()
        );
    }
}

#[test]
fn check_parameter_kinds() {

    lowering_error! {
        program {
            struct Foo<'a> { }
            struct i32 { }
            trait Bar { }
            impl Bar for Foo<i32> { }
        }
        error_msg {
            "incorrect parameter kind: expected lifetime, found type"
        }
    };

    lowering_error! {
        program {
            struct Foo<T> { }
            trait Bar { }
            impl<'a> Bar for Foo<'a> { }
        }
        error_msg {
            "incorrect parameter kind: expected type, found lifetime"
        }
    };

    lowering_error! {
        program {
            trait Iterator { type Item<'a>; }
            trait Foo { }
            impl<X, T> Foo for <X as Iterator>::Item<T> where X: Iterator { }
        }
        error_msg {
            "incorrect kind for associated type parameter: expected lifetime, found type"
        }
    };

    lowering_error! {
        program {
            trait Iterator { type Item<T>; }
            trait Foo { }
            impl<X, 'a> Foo for <X as Iterator>::Item<'a> where X: Iterator { }
        }
        error_msg {
            "incorrect kind for associated type parameter: expected type, found lifetime"
        }
    };

    lowering_error! {
        program {
            trait Into<T> {}
            struct Foo {}
            impl<'a> Into<'a> for Foo {}
        }
        error_msg {
            "incorrect kind for trait parameter: expected type, found lifetime"
        }
    }

    lowering_error! {
        program {
            trait IntoTime<'a> {}
            struct Foo {}
            impl<T> IntoTime<T> for Foo {}
        }
        error_msg {
            "incorrect kind for trait parameter: expected lifetime, found type"
        }
    }
}
