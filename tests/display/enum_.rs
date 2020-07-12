#[test]
fn test_simple_enum() {
    reparse_test!(
        program {
            enum Foo {}
        }
    );
}

#[test]
fn test_enum_generics() {
    reparse_test!(
        program {
            enum Foo<T> {}
            enum Bar<T,B> {}
        }
    );
}

#[test]
fn test_enum_bounds() {
    // Test printing where clauses
    reparse_test!(
        program {
            enum Foo<T> where T: Trait {}
            trait Trait {}
        }
    );
}

#[test]
fn test_enum_fields() {
    // Test printing enums with fields, enum fields with fields, and enum
    // generics in enum fields.
    reparse_test!(
        program {
            enum Foo<T> {}
            enum Bar {}
            enum Baz<T> {
                A {
                    x: Foo<Bar>,
                    b: Bar,
                    y: Foo<T>
                },
                B(u32),
            }
        }
    );
}

#[test]
fn test_enum_keywords() {
    reparse_test!(
        program {
            #[upstream]
            enum UpstreamFoo {}

            #[fundamental]
            enum FundamentalFoo<T> {}

            #[phantom_data]
            enum PhantomFoo {}

            #[upstream]
            #[fundamental]
            #[phantom_data]
            enum Bar<T> {}
        }
    );
}

#[test]
fn test_enum_repr() {
    reparse_test!(
        program {
            #[repr(C)]
            enum CFoo {}

            #[repr(packed)]
            enum PackedFoo {}

            // Test all orderings of multiple `repr()` attributes

            #[repr(C)]
            #[repr(packed)]
            enum CPackedFoo {}

            #[repr(packed)]
            #[repr(C)]
            enum PackedCFoo {}
        }
    );
}

#[test]
fn test_enum_repr_and_keywords_ordered_correctly() {
    // Test that when we print both `repr` and another keyword, we order them in
    // a way accepted by the parser.
    reparse_test!(
        program {
            #[upstream]
            #[repr(C)]
            enum UpstreamCFoo {}
        }
    );
}
