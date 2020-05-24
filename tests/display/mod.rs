#[macro_use]
mod util;

mod assoc_ty;
mod built_ins;
mod dyn_;
mod formatting;
mod impl_;
mod lifetimes;
mod opaque_ty;
mod self_;
mod struct_;
mod trait_;
mod where_clauses;

use self::util::*;

#[test]
fn test_program_writer() {
    reparse_test!(
        program {
            struct Foo { }
            struct Vec<T> { }
            struct Map<_0, _1> { }
            struct Ref<'a, T> { }

            trait Marker { }
            trait Clone { }
            trait Deref<'a, U> {
                type Assoc: Clone;
            }
            trait AssocWithGenerics {
                type Assoc<T>;
            }
            trait AssocTrait3<T> {
                type Assoc<U>;
            }
            trait AsRef<T> { }

            trait AssocTraitWithWhere<T> {
                type Assoc<U> where U: AsRef<T>;
            }

            impl<T> Marker for Vec<T> { }
            impl Clone for Foo { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl<T, U> Clone for Map<T, U> where T: Clone, U: Clone { }

            impl<'a, T, U> Deref<'a, T> for Ref<'a, U> {
                type Assoc = Foo;
            }
            impl AssocWithGenerics for Foo {
                type Assoc<T> = Vec<T>;
            }
            impl<T> AssocTrait3<T> for Vec<T> {
                type Assoc<U> = Map<T, U>;
            }
            impl<T> AssocTraitWithWhere<T> for Vec<T> {
                type Assoc<U> = Map<T, U>;
            }
        }
    );
}
