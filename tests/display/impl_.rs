#[test]
fn test_negative_auto_trait_impl() {
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
fn test_simple_impl() {
    reparse_test!(
        program {
            struct Foo {}
            trait Bar<T> {}
            impl<T> Bar<T> for Foo {}
        }
    );
}

#[test]
fn test_impl_for_generic() {
    reparse_test!(
        program {
            trait Bar<T> {}
            impl<T,G> Bar<T> for G {}
        }
    );
    reparse_test!(
        program {
            trait Baz {}
            impl<T> Baz for T {}
        }
    );
}
