#[test]
fn test_simple_structs_and_bounds() {
    reparse_test!(
        program {
            struct Foo {}
        }
    );
    reparse_test!(
        program {
            struct Foo<T> {}
        }
    );
    reparse_test!(
        program {
            struct Foo<T> where T: Trait {}
            trait Trait {}
        }
    );
}

#[test]
fn test_struct_fields() {
    reparse_test!(
        program {
            struct Foo<T> {}
            struct Bar {}
            struct Baz {
                x: Foo<Bar>,
                b: Bar
            }
        }
    );
}

#[test]
fn test_struct_keywords() {
    reparse_test!(
        program {
            #[upstream]
            struct UpstreamFoo {}
        }
    );
    reparse_test!(
        program {
            #[fundamental]
            struct FundamentalFoo<T> {}
        }
    );
    reparse_test!(
        program {
            #[phantom_data]
            struct PhantomFoo {}
        }
    );
    reparse_test!(
        program {
            #[upstream]
            #[fundamental]
            #[phantom_data]
            struct Bar<T> {}
        }
    );
}
