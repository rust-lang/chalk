#[test]
fn test_forall_in_dyn() {
    reparse_test!(
        program {
            trait Foo<'t> {}
            trait Bar<'a> {}
            impl<'t> Foo<'t> for dyn forall<'a> Bar<'a> + 't {}
        }
    );
    reparse_test!(
        program {
            struct Foo<'t> {
                field: dyn forall<'a> Baz<'a> + 't
            }
            trait Baz<'a> {}
        }
    );
    reparse_test!(
        program {
            trait Foo<'t> {}
            trait Bax<'a, 'b> {}
            impl<'t> Foo<'t> for dyn forall<'a, 'b> Bax<'a, 'b> + 't {}
        }
    );
    reparse_test!(
        program {
            struct Foo<'t> {
                field: dyn forall<'a, 'b> Bix<'a, 'b> + 't
            }
            trait Bix<'a, 'b> {}
        }
    );
    reparse_test!(
        program {
            struct Foo<'t> {
                field: dyn forall<'a> Bex<'a> + forall<'b> Byx<'b> + 't
            }
            trait Bex<'a> {}
            trait Byx<'a> {}
        }
    );
    reparse_test!(
        program {
            struct Foo<'t> {
                field: dyn forall<'a, 'b> Bux<'a, 'b> + forall<'b, 'c> Brx<'b, 'c> + 't
            }
            trait Bux<'a, 'b> {}
            trait Brx<'a, 'b> {}
        }
    );
    reparse_test!(
        program {
            struct Foo<'a> {
                field: dyn forall<'b> Bpx<'a, 'b> + 'a
            }
            trait Bpx<'a, 'b> {}
        }
    );
}

#[test]
fn test_simple_dyn() {
    reparse_test!(
        program {
            struct Foo<'a> {
                field: dyn Bax + 'a
            }
            trait Bax {}
        }
    );
    reparse_test!(
        program {
            struct Foo<'a> {
                field: dyn Bix<'a> + 'a
            }
            trait Bix<'a> {}
        }
    );
}
