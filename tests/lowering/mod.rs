use chalk_integration::db::ChalkDatabase;
use chalk_integration::query::LoweringDatabase;
use chalk_solve::SolverChoice;

#[test]
fn lower_success() {
    lowering_success! {
        program {
            struct Foo { field: Foo }
            trait Bar { }
            impl Bar for Foo { }
        }
    }
}

#[test]
fn not_trait() {
    lowering_error! {
        program {
            struct Foo { }
            trait Bar { }
            impl Foo for Bar { }
        }
        error_msg {
            "expected a trait, found `Foo`, which is not a trait"
        }
    }
}

#[test]
fn auto_trait() {
    lowering_error! {
        program {
            #[auto] trait Foo<T> { }
        }
        error_msg {
            "auto trait `Foo` cannot have parameters"
        }
    }

    lowering_error! {
        program {
            trait Bar { }
            #[auto] trait Foo where Self: Bar { }
        }
        error_msg {
            "auto trait `Foo` cannot have where clauses"
        }
    }

    lowering_error! {
        program {
            #[auto] trait Foo {
                type Item;
            }
        }
        error_msg {
            "auto trait `Foo` cannot define associated types"
        }
    }

    lowering_success! {
        program {
            #[auto] trait Send { }
        }
    }
}

#[test]
fn negative_impl() {
    lowering_error! {
        program {
            trait Foo {
                type Item;
            }

            impl !Foo for i32 {
                type Item = i32;
            }
        }
        error_msg {
            "negative impl for trait `Foo` cannot define associated values"
        }
    }

    lowering_success! {
        program {
            trait Foo { }

            trait Iterator {
                type Item;
            }

            impl<T> !Foo for T where T: Iterator<Item = i32> { }
        }
    }
}

#[test]
fn invalid_name() {
    lowering_error! {
        program {
            struct Foo { }
            trait Bar { }
            impl Bar for X { }
        }
        error_msg {
            "invalid type name `X`"
        }
    }
}

#[test]
fn type_parameter() {
    lowering_success! {
        program {
            struct Foo { }
            trait Bar { }
            impl<X> Bar for X { }
        }
    }
}

#[test]
fn type_parameter_bound() {
    lowering_success! {
        program {
            struct Foo { }
            trait Bar { }
            trait Eq { }
            impl<X> Bar for X where X: Eq { }
        }
    }
}

#[test]
fn assoc_tys() {
    lowering_success! {
        program {
            struct String { }
            struct Char { }

            trait Iterator { type Item; }
            impl Iterator for String { type Item = Char; }

            trait Foo { }
            impl<X> Foo for <X as Iterator>::Item where X: Iterator { }
        }
    }
}

#[test]
fn goal_quantifiers() {
    let db = ChalkDatabase::with("trait Foo<A, B> { }", SolverChoice::default());
    let goal = db
        .parse_and_lower_goal("forall<X> {exists<Y> {forall<Z> {Z: Foo<Y, X>}}}")
        .unwrap();
    db.with_program(|_| {
        assert_eq!(
            format!("{:?}", goal),
            "ForAll<type> { Exists<type> { ForAll<type> { Implemented(^0.0: Foo<^1.0, ^2.0>) } } }"
        );
    });
}

#[test]
fn atc_accounting() {
    let db = ChalkDatabase::with(
        "
            struct Vec<T> { }

            trait Iterable {
                type Iter<'a>;
            }

            impl<T> Iterable for Vec<T> {
                type Iter<'a> = Iter<'a, T>;
            }

            struct Iter<'a, T> { }
            ",
        SolverChoice::default(),
    );
    db.with_program(|program| {
        let atv_text = format!(
            "{:#?}",
            &program.associated_ty_values.values().next().unwrap()
        );
        println!("{}", atv_text);
        assert_eq!(
            &atv_text[..].replace(",\n", "\n"),
            &r#"AssociatedTyValue {
    impl_id: ImplId(#2),
    associated_ty_id: (Iterable::Iter),
    value: for<lifetime, type> AssociatedTyValueBound {
        ty: Iter<'^0.0, ^0.1>
    },
}"#
            .replace(",\n", "\n"),
        );
        let goal = db
            .parse_and_lower_goal(
                "forall<X> { forall<'a> { forall<Y> { \
                 X: Iterable<Iter<'a> = Y> } } }",
            )
            .unwrap();
        let goal_text = format!("{:?}", goal);
        println!("{}", goal_text);
        assert_eq!(
            goal_text,
            "ForAll<type> { \
             ForAll<lifetime> { \
             ForAll<type> { \
             all(AliasEq(<^2.0 as Iterable>::Iter<'^1.0> = ^0.0), \
             Implemented(^2.0: Iterable)) \
             } \
             } \
             }"
        );
    });
}

#[test]
fn check_parameter_kinds() {
    lowering_error! {
        program {
            struct Foo<'a> { }
            struct Myi32 { }
            trait Bar { }
            impl Bar for Foo<Myi32> { }
        }
        error_msg {
            "incorrect parameter kind for `Foo`: expected lifetime, found type"
        }
    };

    lowering_error! {
        program {
            struct Foo<T> { }
            trait Bar { }
            impl<'a> Bar for Foo<'a> { }
        }
        error_msg {
            "incorrect parameter kind for `Foo`: expected type, found lifetime"
        }
    };

    lowering_error! {
        program {
            trait Iterator { type Item<'a>; }
            trait Foo { }
            impl<X, T> Foo for <X as Iterator>::Item<T> where X: Iterator { }
        }
        error_msg {
            "incorrect associated type parameter kind for `Item`: expected lifetime, found type"
        }
    };

    lowering_error! {
        program {
            trait Iterator { type Item<T>; }
            trait Foo { }
            impl<X, 'a> Foo for <X as Iterator>::Item<'a> where X: Iterator { }
        }
        error_msg {
            "incorrect associated type parameter kind for `Item`: expected type, found lifetime"
        }
    };

    lowering_error! {
        program {
            trait Into<T> {}
            struct Foo {}
            impl<'a> Into<'a> for Foo {}
        }
        error_msg {
            "incorrect parameter kind for trait `Into`: expected type, found lifetime"
        }
    }

    lowering_error! {
        program {
            trait IntoTime<'a> {}
            struct Foo {}
            impl<T> IntoTime<T> for Foo {}
        }
        error_msg {
            "incorrect parameter kind for trait `IntoTime`: expected lifetime, found type"
        }
    }
}

#[test]
fn gat_parse() {
    lowering_success! {
        program {
            trait Sized {}
            trait Clone {}

            trait Foo {
                type Item<'a, T>: Sized + Clone where Self: Sized;
            }

            trait Bar {
                type Item<'a, T> where Self: Sized;
            }

            struct Container<T> {
                value: T
            }

            trait Baz {
                type Item<'a, 'b, T>: Foo<Item<'b, T> = Container<T>> + Clone;
            }

            trait Quux {
                type Item<'a, T>;
            }
        }
    }

    lowering_error! {
        program {
            trait Sized { }

            trait Foo {
                type Item where K: Sized;
            }
        }

        error_msg {
            "invalid type name `K`"
        }
    }
}

#[test]
fn gat_higher_ranked_bound() {
    lowering_success! {
        program {
            trait Fn<T> {}
            struct Ref<'a, T> {}
            trait Sized {}

            trait Foo {
                type Item<T>: forall<'a> Fn<Ref<'a, T>> + Sized;
            }
        }
    }
}

#[test]
fn duplicate_parameters() {
    lowering_error! {
        program {
            trait Foo<T, T> { }
        }

        error_msg {
            "duplicate or shadowed parameters"
        }
    }

    lowering_error! {
        program {
            trait Foo<T> {
                type Item<T>;
            }
        }

        error_msg {
            "duplicate or shadowed parameters"
        }
    }

    lowering_error! {
        program {
            struct fun<'a> { }
            struct Foo<'a> {
                a: for<'a> fn(fun<'a>)
            }
        } error_msg {
            "duplicate or shadowed parameters"
        }
    }

    lowering_error! {
        program {
            trait Fn<T> {}
            trait Ref<'a, T> {}

            trait Foo<'a> {
                type Item<T>: forall<'a> Fn<Ref<'a, T>>;
            }
        } error_msg {
            "duplicate or shadowed parameters"
        }
    }
}

#[test]
fn upstream_items() {
    lowering_success! {
        program {
            #[upstream] trait Send { }
            #[upstream] struct Vec<T> { }
        }
    }
}

#[test]
fn fundamental_multiple_type_parameters() {
    lowering_error! {
        program {
            #[fundamental]
            struct Boxes<T, U> { }
        }

        error_msg {
            "only a single parameter supported for fundamental type `Boxes`"
        }
    }
}

#[test]
fn tuples() {
    lowering_success! {
        program {
            trait Foo { }

            // `()` is an empty tuple
            impl Foo for () { }
            // `(i32,)` is a tuple
            impl Foo for (i32,) { }
            // `(i32)` is `i32` is a scalar
            impl Foo for (i32) { }
            impl Foo for (i32, u32) { }
            impl Foo for (i32, u32, f32) { }
        }
    }
}

#[test]
fn scalars() {
    lowering_success! {
        program {
            trait Foo { }

            impl Foo for i8 { }
            impl Foo for i16 { }
            impl Foo for i32 { }
            impl Foo for i64 { }
            impl Foo for i128 { }
            impl Foo for isize { }
            impl Foo for u8 { }
            impl Foo for u16 { }
            impl Foo for u32 { }
            impl Foo for u64 { }
            impl Foo for u128 { }
            impl Foo for usize { }
            impl Foo for f32 { }
            impl Foo for f64 { }
            impl Foo for bool { }
            impl Foo for char { }
        }
    }

    lowering_error! {
        program {
            struct i32 { }
        }

        error_msg {
            "parse error: UnrecognizedToken { token: (8, Token(45, \"i32\"), 11), expected: [\"r#\\\"([A-Za-z]|_)([A-Za-z0-9]|_)*\\\"#\"] }"
        }
    }
}

#[test]
fn tuple_trait_impl() {
    let db = ChalkDatabase::with(
        "
trait Foo { }
struct S1 { }
impl Foo for (S1, S1) { }
",
        SolverChoice::default(),
    );
    let goal = db.parse_and_lower_goal("(S1, S1): Foo").unwrap();
    db.with_program(|_| {
        assert_eq!(format!("{:?}", goal), "Implemented(2<S1, S1>: Foo)");
    });
    let db = ChalkDatabase::with(
        "
trait Foo { }
impl Foo for (i32, i32, (i32,)) { }
",
        SolverChoice::default(),
    );
    let goal = db.parse_and_lower_goal("(i32, i32, (i32,)): Foo").unwrap();
    db.with_program(|_| {
        assert_eq!(
            format!("{:?}", goal),
            "Implemented(3<Int(I32), Int(I32), 1<Int(I32)>>: Foo)"
        );
    });
}

#[test]
fn scalar_trait_impl() {
    let db = ChalkDatabase::with(
        "
trait Foo { }
impl Foo for usize { }
impl Foo for isize { }
impl<T1, T2> Foo for (T1, T2) where T1: Foo, T2: Foo { }
impl<T> Foo for (T,T,T) where T: Foo { }
",
        SolverChoice::default(),
    );
    let goal = db.parse_and_lower_goal("(usize, usize): Foo").unwrap();
    db.with_program(|_| {
        assert_eq!(
            format!("{:?}", goal),
            "Implemented(2<Uint(Usize), Uint(Usize)>: Foo)"
        );
    });
    let goal = db.parse_and_lower_goal("(usize, isize): Foo").unwrap();
    db.with_program(|_| {
        assert_eq!(
            format!("{:?}", goal),
            "Implemented(2<Uint(Usize), Int(Isize)>: Foo)"
        );
    });
    let goal = db.parse_and_lower_goal("(usize, bool): Foo").unwrap();
    db.with_program(|_| {
        // TODO: This should fail (Foo is not implemented for bool)
        assert_eq!(
            format!("{:?}", goal),
            "Implemented(2<Uint(Usize), Bool>: Foo)"
        );
    });
    let goal = db.parse_and_lower_goal("(usize,usize,usize): Foo").unwrap();
    db.with_program(|_| {
        assert_eq!(
            format!("{:?}", goal),
            "Implemented(3<Uint(Usize), Uint(Usize), Uint(Usize)>: Foo)"
        );
    });
    let goal = db.parse_and_lower_goal("(char,u8,i8): Foo").unwrap();
    db.with_program(|_| {
        // TODO: This should fail (the three types are not the same)
        assert_eq!(
            format!("{:?}", goal),
            "Implemented(3<Char, Uint(U8), Int(I8)>: Foo)"
        );
    });
}
