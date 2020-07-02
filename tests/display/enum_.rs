#[test]
fn test_simple_enum_and_bounds() {
    reparse_test!(
        program {
            enum Foo {}
        }
    );
    reparse_test!(
        program {
            enum Foo<T> {}
        }
    );
    reparse_test!(
        program {
            enum Foo<T> where T: Trait {}
            trait Trait {}
        }
    );
}

#[test]
fn test_enum_fields() {
    reparse_test!(
        program {
            enum Foo<T> {}
            enum Bar {}
            enum Baz {
                A {
                    x: Foo<Bar>,
                    b: Bar
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
        }
    );
    reparse_test!(
        program {
            #[fundamental]
            enum FundamentalFoo<T> {}
        }
    );
    reparse_test!(
        program {
            #[phantom_data]
            enum PhantomFoo {}
        }
    );
    reparse_test!(
        program {
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
        }
    );
    reparse_test!(
        program {
            #[repr(packed)]
            enum PackedFoo {}
        }
    );
    reparse_test!(
        program {
            #[repr(C)]
            #[repr(packed)]
            enum CPackedFoo {}
        }
    );
    reparse_test!(
        program {
            #[repr(packed)]
            #[repr(C)]
            enum PackedCFoo {}
        }
    );
    reparse_test!(
        program {
            #[upstream]
            #[repr(C)]
            enum UpstreamCFoo {}
        }
    );
}
