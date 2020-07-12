#[test]
fn test_simple_struct() {
    // Test simplest struct
    reparse_test!(
        program {
            struct Foo {}
        }
    );
}

#[test]
fn test_generic_struct() {
    // Test printing struct generics
    reparse_test!(
        program {
            struct Foo<T> {}
            struct Bar<T, U> {}
        }
    );
}

#[test]
fn test_struct_where_clauses() {
    // Test printing struct where clauses
    reparse_test!(
        program {
            struct Foo<T> where T: Trait {}
            trait Trait {}
        }
    );
}

#[test]
fn test_struct_fields() {
    // Test printing fields in a struct
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
fn test_struct_generic_fields() {
    // Test printing fields which reference a struct's generics
    reparse_test!(
        program {
            struct Foo<'a, T, U> {
                x: (U, T),
                y: &'a (),
            }
        }
    );
}

#[test]
fn test_struct_keywords() {
    // Test each struct keyword, as well as the combination.
    reparse_test!(
        program {
            #[upstream]
            struct UpstreamFoo {}

            #[fundamental]
            struct FundamentalFoo<T> {}

            #[phantom_data]
            struct PhantomFoo {}

            #[upstream]
            #[fundamental]
            #[phantom_data]
            struct Bar<T> {}
        }
    );
}

#[test]
fn test_struct_repr() {
    // Test each struct repr, as well as the combination of two in any ordering.
    reparse_test!(
        program {
            #[repr(C)]
            struct CFoo {}

            #[repr(packed)]
            struct PackedFoo {}

            #[repr(C)]
            #[repr(packed)]
            struct CPackedFoo {}

            #[repr(packed)]
            #[repr(C)]
            struct PackedCFoo {}
        }
    );
}

#[test]
fn test_struct_repr_with_flags() {
    // Test printing both a repr and a flag (to ensure we get the ordering between them right).
    reparse_test!(
        program {
            #[upstream]
            #[repr(C)]
            struct UpstreamCFoo {}
        }
    );
}
