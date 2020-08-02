#[test]
fn test_alias_eq() {
    // Test alias equals bounds (`Third<Assoc = Foo>`) in where clauses.

    // This test uses "produces" as a workaround for the lowering & writing
    // code's behavior. Specifically, `Foo: Bax<T, BaxT=T>` will be transformed
    // into `Foo: Bax<T, BaxT=T>, Foo: Bax<T>`, even if the where clause already
    // includes `Foo: Bax<T>`. The writer code doesn't remove that addition.
    //
    // No matter how many `Foo: Bax<T>` we have in our input, we're always going
    // to get an extra `Foo: Bax<T>` in the output, so they'll never be equal
    // and we need the separate output program.
    reparse_test!(
        program {
            struct Foo { }
            trait Bar { }
            trait Third {
                type Assoc;
            }
            impl Bar for Foo
            where
                Foo: Third<Assoc = Foo>
            {
            }
        }
        produces {
            struct Foo { }
            trait Bar { }
            trait Third {
                type Assoc;
            }
            impl Bar for Foo
            where
                Foo: Third<Assoc = Foo>,
                Foo: Third
            {
            }
        }
    );
    reparse_test!(
        program {
            struct Foo { }
            trait Bar { }
            trait Third {
                type Assoc;
            }
            impl Bar for Foo
            where
                Foo: Third,
                <Foo as Third>::Assoc: Third
            {
            }
        }
    );
}

#[test]
fn test_dyn_on_left() {
    // Test dyn on the left side of a where clause
    // where dyn Bar + 'a: Bar
    //       ^^^^^^^^^^^^
    reparse_test!(
        program {
            struct Foo { }
            trait Bar { }
            trait Baz<'a> {
                type Assoc<T>
                where
                    dyn Bar + 'a: Bar;
            }
            impl<'a> Bar for Foo
            where
                dyn Bar + 'a: Bar
            {
            }
        }
    );
}

#[test]
fn test_generic_vars_inside_assoc_bounds() {
    reparse_test!(
        program {
            struct Foo { }
            trait Bar<T> { }
            trait Baz<'a> {
                type Assoc<T>
                where
                    dyn Bar<T> + 'a: Bar<T>,
                    T: Bar<Foo>,
                    Foo: Bar<T>;
            }
        }
    );
    reparse_test!(
        program {
            struct Foo { }
            trait Bar<T> { }
            trait Baz<'a, U> {
                type Assoc<T>
                where
                    dyn Bar<U> + 'a: Bar<T>,
                    T: Bar<Foo>,
                    Foo: Bar<U>;
            }
        }
    );
    reparse_test!(
        program {
            struct Foo { }
            trait Bar<T, U> { }
            trait Baz<U> {
                type Assoc<T>: Bar<T, U>;
            }
        }
    );
    reparse_test!(
        program {
            struct Foo { }
            trait Bar<T, U> {
                type Quz;
            }
            trait Baz<U> {
                type Zuq<T>: Bar<T, U, Quz=Foo>;
            }
        }
    );
}

#[test]
fn test_complicated_bounds() {
    reparse_test!(
        program {
            struct Foo { }
            trait Bar { }
            trait Baz<T> { }
            trait Bax<T> { type BaxT; }
            trait Test<'a> {
                type Assoc<T>: Bar + Baz<Foo> + Bax<T, BaxT=T>
                    where
                        Foo: Bax<T, BaxT=T>,
                        Foo: Bar,
                        dyn Bar + 'a: Baz<Foo>;
            }
        }
        produces {
            struct Foo { }
            trait Bar { }
            trait Baz<T> { }
            trait Bax<T> { type BaxT; }
            trait Test<'a> {
                type Assoc<T>: Bar + Baz<Foo> + Bax<T, BaxT=T>
                    where
                        Foo: Bax<T, BaxT=T>,
                        Foo: Bax<T>,
                        Foo: Bar,
                        dyn Bar + 'a: Baz<Foo>;
            }
        }
    );
}

#[test]
fn test_struct_where_clauses() {
    reparse_test!(
        program {
            struct Foo<T, U> where T: Baz, U: Bez { }
            trait Baz { }
            trait Bez { }
        }
    );
}

#[test]
fn test_impl_where_clauses() {
    reparse_test!(
        program {
            struct Foo<T, U> where T: Baz, U: Bez { }
            trait Baz { }
            trait Bez { }
            impl<T, U> Bez for Foo<T, U> where T: Baz, U: Bez { }
        }
    );
    // TODO: more of these
}

#[test]
fn test_trait_projection() {
    reparse_test!(
        program {
            struct Flux {}
            struct Foo<T, U> where U: Bez<T>, <U as Bez<T>>::Assoc<Flux>: Baz { }
            trait Baz { }
            trait Bez<T> {
                type Assoc<U>;
            }
        }
    );
}

#[test]
fn test_trait_projection_with_dyn_arg() {
    reparse_test!(
        program {
            struct Foo<'a, T, U> where U: Bez<T>, <U as Bez<T>>::Assoc<dyn Baz + 'a>: Baz { }
            trait Baz { }
            trait Bez<T> {
                type Assoc<U>;
            }
        }
    );
}

#[test]
fn test_forall_in_where() {
    reparse_test!(
        program {
            trait Bax<T> {}
            trait Foo where forall<T> T: Bax<T> {}
        }
    );
    reparse_test!(
        program {
            trait Buz<'a> {}
            trait Foo<T> where forall<'a> T: Buz<'a> {}
        }
    );
    reparse_test!(
        program {
            struct Foo where forall<T> T: Biz {}
            trait Biz {}
        }
    );
    reparse_test!(
        program {
            struct Foo<T> where forall<'a> T: Bez<'a> {}
            trait Bez<'a> {}
        }
    );
}
