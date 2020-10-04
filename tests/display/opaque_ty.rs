#[test]
fn opaque_types() {
    // Test printing opaque type declarations, opaque types in associated types,
    // and opaque types in impls.
    reparse_test!(
        program {
            struct Bar {}
            trait Buz {}
            trait Baz {
                type Hi;
            }
            impl Buz for Bar {}
            impl Baz for Foo {
                type Hi = Foo;
            }
            opaque type Foo: Buz = Bar;
        }
    );
}

#[test]
fn opaque_ty_no_bounds() {
    // Test printing opaque types without any bounds
    reparse_test!(
        program {
            opaque type Foo = ();
        }
    );
}

#[test]
fn test_generic_opaque_types() {
    // Test printing opaque types with generic parameters
    reparse_test!(
        program {
            struct Foo {}
            trait Bar<T> {}
            opaque type Baz<T>: Bar<T> = Foo;

            struct Fou<T> {}
            struct Unit {}
            trait Bau<T, U> {}
            opaque type Boz<U, T>: Bau<Unit, U> = Fou<T>;
        }
    );
}

#[test]
fn test_opaque_type_as_type_value() {
    // Test printing an opaque type as the value for an associated type
    reparse_test!(
        program {
            struct Foo {}
            trait Bar {}
            trait Fuzz {
                type Assoc: Bar;
            }
            impl Bar for Foo {}
            impl Fuzz for Foo {
                type Assoc = Bax;
            }
            opaque type Bax: Bar = Foo;
        }
    );
}

#[test]
fn test_opaque_type_in_fn_ptr() {
    // Test printing an opaque type as the parameter for a fn ptr type
    reparse_test!(
        program {
            struct Foo {}
            trait Bar<T> {}
            trait Faz {
                type Assoc;
            }
            impl Faz for Foo {
                type Assoc = fn(Baz);
            }
            opaque type Baz: Bar<Foo> = Foo;
        }
    );
}

#[test]
fn test_generic_opaque_type_as_value() {
    // Test printing a generic opaque type as an associated type's value
    reparse_test!(
        program {
            struct Foo {}
            trait Bar<T> {}
            trait Fizz {
                type Assoc: Bar<Foo>;
            }
            impl<T> Bar<T> for Foo {}
            impl Fizz for Foo {
                type Assoc = Baz<Foo>;
            }
            opaque type Baz<T>: Bar<T> = Foo;
        }
    );
}

#[test]
fn test_generic_opaque_type_in_fn_ptr() {
    // Test printing a generic opaque type as an fn ptr's parameter
    reparse_test!(
        program {
            struct Foo {}
            trait Bar<T> {}
            trait Faz {
                type Assoc;
            }
            impl Faz for Foo {
                type Assoc = fn(Baz<Foo>);
            }
            impl<T> Bar<T> for Foo { }
            opaque type Baz<T>: Bar<T> = Foo;
        }
    );
}

#[test]
fn multiple_bounds() {
    // Test printing an opaque type with multiple bounds
    reparse_test!(
        program {
            struct Baz {}
            trait Foo {}
            trait Fuu {}
            opaque type Bar: Foo + Fuu = Baz;
        }
    );
}
