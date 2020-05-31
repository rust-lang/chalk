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
