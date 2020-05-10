use super::*;
#[test]
fn test_complicated_bounds() {
    reparse_into_different_test(
        "
            struct Foo { }
            trait Bar { }
            trait Baz<T> { }
            trait Bax<T> { type BaxT; }
            trait Test {
                type Assoc<T>: Bar + Baz<Foo> + Bax<T, BaxT=T>
                    where
                        Foo: Bax<T, BaxT=T>,
                        Foo: Bar,
                        dyn Bar: Baz<Foo>;
            }
            ",
        "
            struct Foo { }
            trait Bar { }
            trait Baz<T> { }
            trait Bax<T> { type BaxT; }
            trait Test {
                type Assoc<T>: Bar + Baz<Foo> + Bax<T, BaxT=T>
                    where
                        Foo: Bax<T, BaxT=T>,
                        Foo: Bax<T>,
                        Foo: Bar,
                        dyn Bar: Baz<Foo>;
            }",
    );
}

#[test]
fn test_struct_where_clauses() {
    reparse_test(
        "
        struct Foo<T, U> where T: Baz, U: Bez { }
        trait Baz { }
        trait Bez { }
        ",
    );
}

#[test]
fn test_impl_where_clauses() {
    reparse_test(
        "
        struct Foo<T, U> where T: Baz, U: Bez { }
        trait Baz { }
        trait Bez { }
        impl<T, U> Bez for Foo<T, U> where T: Baz, U: Bez { }
        ",
    );
    // TODO: more of these
}

#[test]
fn test_trait_projection() {
    reparse_test(
        "
        struct Flux {}
        struct Foo<T, U> where U: Bez<T>, <U as Bez<T>>::Assoc<Flux>: Baz { }
        trait Baz { }
        trait Bez<T> {
            type Assoc<U>;
        }
        ",
    );
}
#[test]
fn test_trait_projection_with_dyn_arg() {
    reparse_test(
        "
        struct Foo<T, U> where U: Bez<T>, <U as Bez<T>>::Assoc<dyn Baz>: Baz { }
        trait Baz { }
        trait Bez<T> {
            type Assoc<U>;
        }
        ",
    );
}

#[test]
fn test_forall_in_where() {
    reparse_test(
        "
            trait Bax<T> {}
            trait Foo where forall<T> T: Bax<T> {}
            ",
    );
    reparse_test(
        "
            trait Buz<'a> {}
            trait Foo<T> where forall<'a> T: Buz<'a> {}
            ",
    );
    reparse_test(
        "
            struct Foo where forall<T> T: Biz {}
            trait Biz {}
            ",
    );
    reparse_test(
        "
            struct Foo<T> where forall<'a> T: Bez<'a> {}
            trait Bez<'a> {}
            ",
    );
}
