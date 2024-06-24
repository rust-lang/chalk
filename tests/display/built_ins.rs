use super::*;

#[test]
fn test_function_pointer_type() {
    // Test we can print the `fn()` type at all. (impl blocks are simply used as
    // a way to reference this concrete type conveniently)
    reparse_test!(
        program {
            struct Foo { }
            trait Baz<T> { }
            impl Baz<fn(Foo)> for Foo { }
            impl Baz<Foo> for fn(Foo) { }
        }
    );
}

#[test]
fn test_generic_function_pointer_type() {
    // Test we can print a `fn()` type which references generics introduced in
    // outer scopes.
    reparse_test!(
        program {
            struct Foo<'a, T>
            {
                bar: fn(&'a T) -> &'a (T, T)
            }
        }
    );
}

#[test]
fn test_scalar_types() {
    // This is intended to test every scalar in a variety of places. In other
    // words, test the matrix of {every scalar} x {concrete type usages}. This
    // test should be updated to include new scalars, but it isn't super
    // important that it includes every place a concrete type can be used.
    let basic = &["bool", "char", "f16", "f32", "f64", "f128"];
    let ints = {
        let prefixes = &["u", "i"];
        let sizes = &["size", "8", "16", "32", "64", "128"];
        prefixes
            .iter()
            .flat_map(move |&p| sizes.iter().map(move |&size| format!("{}{}", p, size)))
    };
    let basic = basic.iter().copied().map(str::to_owned);

    let scalars = basic.chain(ints);

    for scalar in scalars {
        reparse_test(&format!(
            "
                struct Foo {{
                    field: {0}
                }}
                trait Bar {{
                    type Baz;
                }}
                impl Bar for Foo {{
                    type Baz = {0};
                }}
                impl Bar for {0} {{
                    type Baz = {0};
                }}
                ",
            scalar
        ));
    }
}

#[test]
fn test_slice_types() {
    // Test that we print slice types correctly in a variety of places.
    reparse_test!(
        program {
            struct Foo<T> {
                field: [T]
            }
            trait Bar {
                type Baz;
            }
            impl<T> Bar for Foo<T> {
                type Baz = [T];
            }
            impl<T> Bar for [T] {
                type Baz = Foo<T>;
            }
        }
    );
}

#[test]
fn test_str_types() {
    // Test that we print 'str' correctly in a variety of places.
    reparse_test!(
        program {
            struct Foo {
                field: str
            }
            trait Bar {
                type Baz;
            }
            impl Bar for Foo {
                type Baz = str;
            }
            impl Bar for str {
                type Baz = str;
            }
        }
    );
}

#[test]
fn test_const_ptr() {
    // Test that we can print *const ptrs in various places, including with generics.
    reparse_test!(
        program {
            struct Foo<T> {
                field: *const T
            }
            trait Bar {
                type Baz;
            }
            impl<T> Bar for Foo<T> {
                type Baz = *const u32;
            }
            impl Bar for *const u32 {
                type Baz = *const u32;
            }
            impl<T> Bar for *const T {
                type Baz = *const T;
            }
        }
    );
}

#[test]
fn test_mut_ptr() {
    // Test that we can print *mut ptrs in various places, including with generics.
    reparse_test!(
        program {
            struct Foo<T> {
                field: *mut T
            }
            trait Bar {
                type Baz;
            }
            impl<T> Bar for Foo<T> {
                type Baz = *mut u32;
            }
            impl Bar for *mut u32 {
                type Baz = *mut u32;
            }
            impl<T> Bar for *mut T {
                type Baz = *mut T;
            }
        }
    );
}

#[test]
fn test_immutable_references() {
    reparse_test!(
        program {
            struct Foo<'a,T> {
                field: &'a T
            }
            trait Bar {
                type Baz;
            }
            impl<'a,T> Bar for Foo<'a,T> {
                type Baz = &'a u32;
            }
            impl<'a> Bar for &'a u32 {
                type Baz = &'a u32;
            }
            impl<'a,T> Bar for &'a T {
                type Baz = &'a T;
            }
        }
    );
}

#[test]
fn test_mutable_references() {
    reparse_test!(
        program {
            struct Foo<'a,T> {
                field: &'a mut T
            }
            trait Bar {
                type Baz;
            }
            impl<'a,T> Bar for Foo<'a,T> {
                type Baz = &'a mut u32;
            }
            impl<'a> Bar for &'a mut u32 {
                type Baz = &'a u32;
            }
            impl<'a,T> Bar for &'a mut T {
                type Baz = &'a mut T;
            }
        }
    );
}

#[test]
fn test_empty_tuple() {
    // Test empty tuples print correctly
    reparse_test!(
        program {
            struct Fuu {
                fuu_field: ()
            }
        }
    );
}

#[test]
fn test_one_and_many_tuples() {
    // Test that single-element tuple is printed correctly with the required
    // trailing comma that differentiates it from a parenthesized expression
    reparse_test!(
        program {
            struct Uff {
                fuu_field: (Iff,),
                iff2_field: (Iff, Iff, Iff)
            }
            struct Iff { }
        }
    );
}

#[test]
fn test_tuples_using_generic_args() {
    // Test 1, many tuples which contain generic parameters.
    reparse_test!(
        program {
            struct Foo<T> {
                field: (u32,*const T,T),
                field2: (T,),
                field3: (T)
            }
            trait Bar {
                type Baz;
            }
            impl<T> Bar for Foo<T> {
                type Baz = (T,Foo<T>,u32);
            }
        }
    );
}

#[test]
fn test_impl_on_tuples_with_generics() {
    // Test 0, 1, many tuples in one more place - impl blocks.
    reparse_test!(
        program {
            trait Blug {}
            impl<T1,T2> Blug for (T1,T2) {

            }
            impl<T1> Blug for (T1,) {

            }
            impl Blug for () {

            }
        }
    );
}

#[test]
fn test_array_types() {
    // Test that we print array types correctly in multiple places they can occur.
    reparse_test!(
        program {
            struct Bazz { }
            struct Bar<const T> {
                field: [Bazz; T]
            }
            trait Foo { }
            impl<const T> Foo for [Bazz; T] { }
        }
    );
}
