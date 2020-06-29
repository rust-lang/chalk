#[test]
fn test_various_forall() {
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
    reparse_test!(
        program {
            struct Foo<'a, 'b>
            where
                'a: 'b
            { }

            trait Baz<'a,'b>
            where
                'a: 'b
            { }

            impl<'a,'b,'c> Baz<'a,'b> for Foo<'a,'c>
            where
                'a: 'c,
                'b: 'c
            { }
        }
    );
}
