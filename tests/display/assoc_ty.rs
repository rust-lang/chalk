use super::*;
#[test]
fn test_trait_impl_associated_type() {
    reparse_test(
        "
        struct Foo { }
        struct Floo { }
        trait Bar {
            type Assoc;
        }
        impl Bar for Foo {
            type Assoc = Floo;
        }
        ",
    );
    reparse_test(
        "
        struct Foo { }
        struct Floo { }
        trait Bax {
            type Assoc1;
            type Assoc2;
        }
        impl Bax for Foo {
            type Assoc1 = Floo;
            type Assoc2 = Foo;
        }
        ",
    );
}

#[test]
fn test_trait_impl_associated_type_with_generics() {
    reparse_test(
        "
        struct Foo { }
        struct Floo<T> { }
        trait Baz<T> {
            type Assoc;
        }
        impl<T> Baz<T> for Foo {
            type Assoc = Floo<T>;
        }
        ",
    );
    reparse_test(
        "
        struct Foo { }
        struct Floo<T> { }
        trait Bur {
            type Assoc<A>;
        }
        impl Bur for Foo {
            type Assoc<A> = Floo<A>;
        }
        ",
    );
    reparse_test(
        "
        struct Foo { }
        struct Floo<T, U> { }
        trait Bun<T> {
            type Assoc<A>;
        }
        impl<T, U> Bun<T> for Foo {
            type Assoc<A> = Floo<T, A>;
        }
        ",
    );
    reparse_test(
        "
        struct Foo { }
        struct Floo<T, U> { }
        trait Bun<T, U> {
            type Assoc1<A, N>;
            type Assoc2<B>;
            type Assoc3<C, D>;
        }
        impl<T, U> Bun<T, U> for Foo {
            type Assoc1<A, N> = Floo<N, T>;
            type Assoc2<B> = Floo<U, B>;
            type Assoc3<C, D> = Floo<Floo<T, D>, Floo<U, C>>;
        }
        ",
    );
}
#[test]
fn test_simple_assoc_type() {
    reparse_test(
        "
            trait Foo {
                type Assoc;
            }
            ",
    );
    reparse_test(
        "
            trait Byz {}
            trait Buzz {}
            trait Foo {
                type Assoc: Byz + Buzz;
            }
            ",
    );
}

#[test]
fn test_simple_generic_assoc_type() {
    reparse_test(
        "
            trait Trait {}
            trait Foo {
                type Assoc<Y>;
            }
            ",
    );
    reparse_test(
        "
            trait Trait {}
            trait Foo {
                type Assoc<Y>: Trait;
            }
            ",
    );
    reparse_test(
        "
            trait Trait {}
            trait Foo {
                type Assoc<Y> where Y: Trait;
            }
            ",
    );
}

#[test]
fn test_assoc_type_in_generic_trait() {
    reparse_test(
        "
            trait Foo<T> {
                type Assoc;
            }
            ",
    );
    reparse_test(
        "
            trait Fou<T, U, F> {
                type Assoc;
            }
            ",
    );
    reparse_test(
        "
            trait Bax {}
            trait Foo<T> {
                type Assoc where T: Bax;
            }
            ",
    );
    reparse_test(
        "
            trait Bix<T> {}
            trait Foo<T> {
                type Assoc<Y> where Y: Bix<T>;
            }
            ",
    );
}

#[test]
fn test_impl_assoc_ty() {
    reparse_test(
        "
            struct Fuu {}
            trait Bhz {
                type Assoc;
            }
            impl Bhz for Fuu {
                type Assoc = Fuu;
            }
            ",
    );
    reparse_test(
        "
            struct Fou {}
            trait Bax<T> {
                type Assoc;
            }
            impl<T> Bax<T> for Fou {
                type Assoc = Fou;
            }
            ",
    );
    reparse_test(
        "
            struct Fuu {}
            trait Bmx<T> {
                type Assoc;
            }
            impl<T> Bmx<T> for Fuu {
                type Assoc = T;
            }
            ",
    );
    reparse_test(
        "
            struct Fuu {}
            struct Guu<T> {}
            trait Bmx<T> {
                type Assoc;
            }
            impl<T> Bmx<T> for Fuu {
                type Assoc = Guu<T>;
            }
            ",
    );
    reparse_test(
        "
            struct Fuu {}
            struct Guu<T, U> {}
            trait Bmx<T> {
                type Assoc<U>;
            }
            impl<T> Bmx<T> for Fuu {
                type Assoc<U> = Guu<T, U>;
            }
            ",
    );
}

#[test]
fn test_impl_assoc_ty_alias() {
    reparse_test(
        "
        struct Fow {}
        struct Qac {}
        trait Bow<T> {}
        trait Baq<T> {
            type Assoc<G>: Boo<G, Item=Fow>;
        }
        trait Boo<T> {
            type Item;
        }
        impl<T> Boo<T> for Qac {
            type Item = Fow;
        }
        impl<T> Baq<T> for Fow {
            type Assoc<U> = Qac;
        }
    ",
    );
}

#[test]
fn test_assoc_ty_alias_bound() {
    // Foo: Bax<BaxT=T> is lowered into two bounds, Bax and Bax<BaxT=T>, and
    // the formatter does not coalesce those bounds.
    reparse_into_different_test(
        "
        struct Foo { }
        trait Bax { type BaxT; }
        trait Test {
            type Assoc<T>
                where
                    Foo: Bax<BaxT=T>;
        }
        ",
        "
        struct Foo { }
        trait Bax { type BaxT; }
        trait Test {
            type Assoc<T>
                where
                    Foo: Bax<BaxT=T>,
                    Foo: Bax;
        }
        ",
    );
    reparse_into_different_test(
        "
        struct Foo<T> where T: Baux<Assoc=T> { }
        trait Baux { type Assoc; }
        ",
        "
        struct Foo<T> where T: Baux<Assoc=T>, T: Baux { }
        trait Baux { type Assoc; }
        ",
    );
    reparse_into_different_test(
        "
        struct Foo<T> {}
        trait Boz { type Assoc; }
        impl<T> Boz for Foo<T> where T: Boz<Assoc=Foo<T>> {
            type Assoc = Foo<T>;
        }
        ",
        "
        struct Foo<T> {}
        trait Boz { type Assoc; }
        impl<T> Boz for Foo<T> where T: Boz<Assoc=Foo<T>>, T: Boz {
            type Assoc = Foo<T>;
        }
        ",
    );
}
