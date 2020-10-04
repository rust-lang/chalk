#[test]
fn test_basic_fn_def() {
    // Test printing simple function definitions
    reparse_test!(
        program {
            struct Foo {}
            fn nothing();
            fn takes_foo(v: Foo);
            fn gives_foo() -> Foo;
            fn bar(a: Foo, _: Foo) -> Foo;
        }
    );
}

#[test]
fn test_generic_fn_def() {
    // Test printing generics in function definitions
    reparse_test!(
        program {
            struct Foo<T> {}
            struct Bar<T> {}
            fn identity<T>(arg: T) -> T;
            fn transform<T>(a: Foo<T>) -> Bar<T>;
            fn wrap<T>(v: T) -> Foo<T>;
        }
    );
}

#[test]
fn test_const_generic_fn_def() {
    // Test printing const generics in function definitions
    reparse_test!(
        program {
            fn uses_n<T, const N>(arg: [T; N]);
        }
    );
}

#[test]
fn test_opaque_ty_with_fn_def() {
    // Test printing opaque types in function definitions
    reparse_test!(
        program {
            opaque type Bar = ();
            fn gives_bar() -> Bar;
            fn receives_bar(param: Bar) -> ();
        }
    );
}

// These `test_fn_as_type_*` tests test various usages of fn types

// We do not yet support "fn def" types, which this uses.
#[test]
#[ignore]
fn test_fn_as_type_in_functions() {
    //(TODO: cover remaining places when functionality is implemented)

    // Test printing an fn type in a function definitions parameters and return
    // type.
    reparse_test!(
        program {
            fn foo(arg: u32);
            fn baz(foo) -> u32;
            fn bar() -> foo;
        }
    );
}

// We do not yet support "fn def" types, which this uses.
#[test]
#[ignore]
fn test_fn_as_type_in_opaque_ty_value() {
    // Test printing an fn type as an opaque type's hidden value
    reparse_test!(
        program {
            trait Bar {}
            fn foo();
            impl Bar for Foo {}
            opaque type Zed: Bar = foo;
        }
    );
}

// We do not yet support "fn def" types, which this uses.
#[test]
#[ignore]
fn test_fn_as_type_in_struct_field() {
    // Test printing an fn type as a struct type's field
    reparse_test!(
        program {
            fn foo();
            struct Vi {
                field: foo
            }
        }
    );
}
