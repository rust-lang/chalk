use super::*;

#[test]
fn test_simple_structs_and_bounds() {
    reparse_test("struct Foo {}");
    reparse_test("struct Foo<T> {}");
    reparse_test(
        "
                struct Foo<T> where T: Trait {}
                trait Trait {}
                ",
    );
}

#[test]
fn test_struct_fields() {
    reparse_test(
        "
                struct Foo<T> {}
                struct Bar {}
                struct Baz {
                    x: Foo<Bar>,
                    b: Bar
                }
                ",
    );
}
