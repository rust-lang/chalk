#[test]
fn test_basic_fn_def() {
    reparse_test!(
        program {
            fn nothing();
        }
    );
    reparse_test!(
        program {
            struct Foo {}
            fn takes_foo(v: Foo);
            fn gives_foo() -> Foo;
            fn bar(a: Foo, _: Foo) -> Foo;
        }
    );
}

#[test]
fn test_generic_fn_def() {
    reparse_test!(
        program {
            fn identity<T>(arg: T) -> T;
        }
    );
    reparse_test!(
        program {
            struct Foo<T> {}
            struct Bar<T> {}
            fn transform<T>(a: Foo<T>) -> Bar<T>;
            fn wrap<T>(v: T) -> Foo<T>;
        }
    );
}

#[test]
fn test_const_generic_fn_def() {
    reparse_test!(
        program {
            fn uses_n<T, const N>(arg: [T; N]);
        }
    );
}

#[test]
fn test_opaque_ty_with_fn_def() {
    reparse_test!(
        program {
            opaque type Bar: = ();
            fn gives_bar() -> Bar;
        }
    );
}

/// We do not yet support "fn def" types, which this uses.
#[test]
#[ignore]
fn test_fn_as_type() {
    reparse_test!(
        program {
            fn foo(arg: u32);
            fn bar() -> foo;
        }
    );
    reparse_test!(
        program {
            trait Bar {}
            fn foo();
            impl Bar for Foo {}
            opaque type Zed: Bar = foo;
        }
    );
    reparse_test!(
        program {
            fn foo();
            struct Vi {
                field: foo
            }
        }
    );
}
