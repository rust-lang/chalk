#[test]
fn test_self_in_trait_bounds() {
    // Test 'Self' printing in trait where clauses.
    reparse_test!(
        program {
            trait Bkz {}
            trait Foo where Self: Bkz {}
        }
    );
}

#[test]
fn test_self_in_forall() {
    // Test 'Self' printing inside a forall clause.
    reparse_test!(
        program {
            trait Baz<'a> {}
            trait Foo where forall<'a> Self: Baz<'a> {}
        }
    );
}

#[test]
fn test_self_in_assoc_type_declarations() {
    // Test 'Self' in associated types declarations prints correctly.
    reparse_test!(
        program {
            trait Extra<T> {}
            trait Bez {}
            trait Foo {
                type Assoc1: Extra<Self>;
                type Assoc2 where Self: Bez;
            }
        }
    );
}

#[test]
fn test_self_in_generic_associated_type_declarations() {
    // Test 'Self' in generic associated type declarations prints correctly.
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
    // Test that 'Self' in dyn correctly refers to the outer Self correctly.
    reparse_test!(
        program {
            trait Bun<T> {}
            trait Foo<'a, T> {
                type Assoc<U> where dyn Bun<Self> + 'a: Bun<U>;
            }
        }
    );
}

#[test]
fn test_self_in_dyn_with_generics() {
    // Test that 'Self' in dyn correctly refers to the outer Self when in the
    // presence of generics introduced at the same time as that Self. In
    // addition, test those generics also print correctly inside `dyn`.
    reparse_test!(
        program {
            trait Has<T> {}
            trait Bun<T, U> {}
            trait Fiz<'a, T> {
                type Assoc1<U>: Has<dyn Bun<Self, U> + 'a>;
                type Assoc2<N>: Has<dyn Bun<T, Self> + 'a>;
            }
        }
    );
}

// Self doesn't work in these circumstances yet (test programs fail to lower)
#[ignore]
#[test]
fn test_self_in_struct_bounds() {
    // Test 'self' prints correctly in various places in struct where clauses.
    reparse_test!(
        program {
            trait Bax<T> {}
            trait Baz {}
            struct Foo<T>
            where
                T: Bax<Self>,
                Self: Baz
            {
            }
        }
    );
}

// Self doesn't work in these circumstances yet (test programs fail to lower)
#[ignore]
#[test]
fn test_self_in_impl_block_associated_types() {
    // Test 'Self' prints correctly in associated type values.
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
}
// Self doesn't work in these circumstances yet (test programs fail to lower)
#[ignore]
#[test]
fn test_self_in_impl_block_associated_type_with_generics() {
    // Test 'Self' prints correctly in impl blocks where we introduce
    // generic parameters. In addition, test those parameters print correctly
    // next to self.
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

// Self doesn't work in these circumstances yet (test programs fail to lower)
#[ignore]
#[test]
fn test_self_in_impl_block_where_clauses() {
    // Test 'Self' prints correctly in impl block where clauses.
    reparse_test!(
        program {
            trait Foo {}
            trait Fin {}
            struct Bux {}
            impl Foo for Bux where Self: Fin {}
        }
    );
}

#[test]
fn test_against_accidental_self() {
    // In some of the writer code, it would be really easy to accidentally
    // outputs the first generic parameter of an item as "Self". This tests
    // against that.
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
