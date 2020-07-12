#[test]
fn test_various_forall() {
    // Test we print lifetime vars introduced by 'forall' in various situations.
    reparse_test!(
        program {
            struct Foo<'b> where forall<'a> Foo<'a>: Baz<'a> { }
            trait Baz<'a> {}
            trait Bax<'a> {}
            trait Biz {
                type Bex: forall<'a> Bax<'a>;
            }
            impl<'a> Baz<'a> for for<'b> fn(Foo<'b>) { }
            impl<'a> Bax<'a> for fn(Foo<'a>) { }
            impl<'a> Bax<'a> for dyn forall<'b> Baz<'b> + 'a { }
        }
    );
}

#[test]
fn test_lifetimes_in_structs() {
    // Test printing lifetimes introduced by structs.
    reparse_test!(
        program {
            struct Foo<'b> { }
            trait Baz<'a> {}
            impl<'a> Baz<'a> for Foo<'a> { }
        }
    );
}

#[test]
fn test_lifetime_outlives() {
    // Test printing lifetime outlives where clauses in a few places they can appear.
    reparse_test!(
        program {
            struct Foo<'a, 'b>
            where
                'a: 'b
            { }

            trait Baz<'a, 'b>
            where
                'a: 'b
            { }

            impl<'a, 'b, 'c> Baz<'a, 'b> for Foo<'a, 'c>
            where
                'a: 'c,
                'b: 'c
            { }
        }
    );
}
