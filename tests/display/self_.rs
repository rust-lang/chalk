#[test]
fn test_self_in_trait_where() {
    reparse_test!(
        program {
            trait Bkz {}
            trait Foo where Self: Bkz {}
        }
    );
    reparse_test!(
        program {
            trait Baz<'a> {}
            trait Foo where forall<'a> Self: Baz<'a> {}
        }
    );
}

#[test]
fn test_self_in_assoc_type() {
    reparse_test!(
        program {
            trait Extra<T> {}
            trait Bez {}
            trait Foo {
                type Assoc: Extra<Self>;
            }
        }
    );

    reparse_test!(
        program {
            trait Bez {}
            trait Foo {
                type Assoc where Self: Bez;
            }
        }
    );
    reparse_test!(
        program {
            trait Biz<T, U, V> {}
            trait Foo<A> {
                type Assoc<B> where Self: Biz<Self, A, B>;
            }
        }
    );
}

#[test]
fn test_self_in_dyn() {
    reparse_test!(
        program {
            trait Bun<T> {}
            trait Foo<T> {
                type Assoc<U> where dyn Bun<Self>: Bun<U>;
            }
        }
    );
    reparse_test!(
        program {
            trait Has<T> {}
            trait Bun<T, U> {}
            trait Fiz<T> {
                type Assoc1<U>: Has<dyn Bun<Self, U>>;
                type Assoc2<N>: Has<dyn Bun<T, Self>>;
            }
        }
    );
}

// Self doesn't work in these circumstances yet (test programs fail to lower)
#[ignore]
#[test]
fn test_self_in_struct_bounds() {
    reparse_test!(
        program {
            trait Bax<T> {}
            struct Foo<T> where T: Bax<Self> {}
        }
    );
    reparse_test!(
        program {
            trait Baz {}
            struct Foo where Self: Baz {}
        }
    );
    reparse_test!(
        program {
            trait Blz {}
            struct Foo<T> where Self: Blz {}
        }
    );
}

// Self doesn't work in these circumstances yet (test programs fail to lower)
#[ignore]
#[test]
fn test_self_in_impl_blocks() {
    reparse_test!(
        program {
            trait Foo {
                type Assoc;
            }
            struct Bix {}
            impl Foo for Bix {
                type Assoc = Self;
            }
        }
    );
    reparse_test!(
        program {
            trait Foo {}
            trait Fin {}
            struct Bux {}
            impl Foo for Bux where Self: Fin {}
        }
    );
    reparse_test!(
        program {
            trait Faux<T, U> {}
            trait Paw<T> {
                type Assoc1<U>;
                type Assoc2<N>;
            }
            struct Buzz {}
            impl<T> Paw<T> for Buzz {
                type Assoc1<U> = dyn Faux<Self, U>;
                type Assoc2<N> = dyn Faux<T, Self>;
            }
        }
    );
}

#[test]
fn test_against_accidental_self() {
    // In some of the writer code, it would be really easy to introduce a
    // outputs the first generic parameter of things as "Self".
    let in_structs = reparse_test!(
        program {
            struct Foo<T> {
                field: T
            }
        }
    );
    assert!(!in_structs.output_text.contains("Self"));
    let in_impl = reparse_test!(
        program {
            struct Foo<T> {}
            trait Bux<U> {
                type Assoc;
            }
            impl<T> Bux<T> for Foo<T> {
                type Assoc = T;
            }
        }
    );
    assert!(!in_impl.output_text.contains("Self"));
    let in_opaque = reparse_test!(
        program {
            struct Foo<T> {}
            trait Que<T> {}
            impl<T> Que<T> for Foo<T> {}
            opaque type Bar<T>: Que<T> = Foo<T>;
        }
    );
    assert!(!in_opaque.output_text.contains("Self"));
}
