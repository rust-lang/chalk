#[test]
fn test_const_generics() {
    reparse_test!(
        program {
            struct Usize { }
            struct Bar<T, const U> { }
            trait Foo<T, const U> { }
            trait AssocTy<const T> {
                type Type<const P>;
            }
            impl<T, const U> Foo<T,U> for Bar<T,U> { }
            impl<T, const U> AssocTy<U> for Bar<T,U> {
                type Type<const P> = Usize;
            }
            opaque type Gee<const G,T,const U>: Foo<T,U> = Usize;
        }
    );
}

#[test]
#[ignore]
fn test_basic_const_values() {
    reparse_test!(
        program {
            struct Foo<const N> { }
            trait Bar { }
            impl Bar for Foo<0> { }
            impl Bar for Foo<1> { }
            impl Bar for Foo<2> { }
        }
    );
    reparse_test!(
        program {
            struct Foo<const N> { }
            opaque type Zed: = Foo<0>;
        }
    );
    reparse_test!(
        program {
            struct Foo<const N> { }
            trait Bar {
                type Assoc;
            }
            impl Bar for Foo<0> {
                type Assoc = Foo<1>;
            }
        }
    );
}
