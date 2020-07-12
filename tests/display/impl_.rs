#[test]
fn test_negative_auto_trait_impl() {
    // Test we can render negative impls.
    reparse_test!(
        program {
            struct Foo { }
            #[auto]
            trait Baz {}
            impl !Baz for Foo { }
        }
    );
}

#[test]
fn test_generic_impl() {
    // Tests we can print generics in impl blocks
    reparse_test!(
        program {
            trait Baz {}
            impl<T> Baz for T {}
        }
    );
}

#[test]
fn test_impl_for_generic_adt() {
    // Test that we can refer to impl-introduced generics in the impl decl
    reparse_test!(
        program {
            trait Bar<T> {}
            impl<T,G> Bar<T> for G {}
        }
    );
}

#[test]
fn test_upstream_impl_keyword() {
    // Test we print the "upstream" keyword.
    reparse_test!(
        program {
            struct Bar {}
            trait Foo {}
            #[upstream]
            impl Foo for Bar {}
        }
    );
}
