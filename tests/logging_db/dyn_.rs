use super::*;
#[test]
fn test_forall_in_dyn() {
    reparse_test(
        "
                trait Foo {}
                trait Bar<'a> {}
                impl Foo for dyn forall<'a> Bar<'a> {}
                ",
    );
    reparse_test(
        "
                struct Foo {
                    field: dyn forall<'a> Baz<'a>
                }
                trait Baz<'a> {}
                ",
    );
    reparse_test(
        "
                trait Foo {}
                trait Bax<'a, 'b> {}
                impl Foo for dyn forall<'a, 'b> Bax<'a, 'b> {}
                ",
    );
    reparse_test(
        "
                struct Foo {
                    field: dyn forall<'a, 'b> Bix<'a, 'b>
                }
                trait Bix<'a, 'b> {}
                ",
    );
    reparse_test(
        "
                struct Foo {
                    field: dyn forall<'a> Bex<'a> + forall<'b> Byx<'b>
                }
                trait Bex<'a> {}
                trait Byx<'a> {}
                ",
    );
    reparse_test(
        "
                struct Foo {
                    field: dyn forall<'a, 'b> Bux<'a, 'b> + forall<'b, 'c> Brx<'b, 'c>
                }
                trait Bux<'a, 'b> {}
                trait Brx<'a, 'b> {}
                ",
    );
    reparse_test(
        "
                struct Foo<'a> {
                    field: dyn forall<'b> Bpx<'a, 'b>
                }
                trait Bpx<'a, 'b> {}
                ",
    );
}

#[test]
fn test_simple_dyn() {
    reparse_test(
        "
                struct Foo {
                    field: dyn Bax
                }
                trait Bax {}
                ",
    );
    reparse_test(
        "
                struct Foo<'a> {
                    field: dyn Bix<'a>
                }
                trait Bix<'a> {}
                ",
    );
}
