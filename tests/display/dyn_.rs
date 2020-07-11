#[test]
fn test_dyn_forall_in_impl() {
    // Test we render `dyn forall` types correctly (and with the correct
    // lifetime names) in impl blocks.
    reparse_test!(
        program {
            trait Foo<'t> {}
            trait Bar<'a> {}
            impl<'t> Foo<'t> for dyn forall<'a> Bar<'a> + 't {}
        }
    );
}

#[test]
fn test_dyn_forall_in_struct() {
    // Test we render `dyn forall` types correctly (and with the correct
    // lifetime names) in struct fields.
    reparse_test!(
        program {
            struct Foo<'t> {
                field: dyn forall<'a> Baz<'a> + 't
            }
            trait Baz<'a> {}
        }
    );
}

#[test]
fn test_dyn_forall_multiple_parameters() {
    // Test we render `dyn forall` types with multiple lifetimes correctly, and
    // with the correct lifetime names.
    reparse_test!(
        program {
            struct Foo<'t> {
                field: dyn forall<'a, 'b> Bix<'a, 'b> + 't
            }
            trait Bix<'a, 'b> {}
        }
    );
}

#[test]
fn test_multiple_forall_one_dyn() {
    // Test we render `dyn forall A + forall B` correctly.
    reparse_test!(
        program {
            struct Foo<'t> {
                field1: dyn forall<'a> Bex<'a> + forall<'b> Byx<'b> + 't,
                field2: dyn forall<'a, 'b> Bux<'a, 'b> + forall<'b, 'c> Brx<'b, 'c> + 't
            }
            trait Bex<'a> {}
            trait Byx<'a> {}
            trait Bux<'a, 'b> {}
            trait Brx<'a, 'b> {}
        }
    );
}

#[test]
fn test_dyn_forall_with_trait_referencing_outer_lifetime() {
    // Test we can render a trait inside a `dyn forall` referencing an outer
    // lifetime correctly (in other words, test for debrujin index errors).
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
    // Test that we print `dyn Trait` types correctly.
    reparse_test!(
        program {
            struct Foo<'a> {
                field: dyn Bax + 'a
            }
            trait Bax {}
        }
    );
}

#[test]
fn test_simple_dyn_referencing_outer_generic_parameters() {
    // Test that we print `dyn Trait` referencing outer generic parameters correctly.
    reparse_test!(
        program {
            struct Foo<'a> {
                field: dyn Bix<'a> + 'a
            }
            trait Bix<'a> {}
        }
    );
}
