use super::*;
#[test]
fn test_function_type() {
    reparse_test!(
        program {
            struct Foo { }
            trait Baz<T> { }
            impl Baz<fn(Foo)> for Foo { }
        }
    );
    reparse_test!(
        program {
            struct Foo { }
            trait Baz { }
            impl Baz for fn(Foo) { }
        }
    );
}

#[test]
fn test_function_type_generic() {
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
    let basic = vec!["bool", "char", "f32", "f64"];
    let sizes = vec!["size", "8", "16", "32", "64", "128"];
    let prefixes = vec!["u", "i"];
    let ints = prefixes
        .iter()
        .flat_map(|&p| sizes.iter().map(move |&size| format!("{}{}", p, size)));
    let scalars = basic.iter().copied().map(str::to_owned).chain(ints);

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
fn test_raw_ptr_types() {
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
        }
    );
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
        }
    );
    reparse_test!(
        program {
            trait Bar {
                type Baz;
            }
            impl<T> Bar for *mut T {
                type Baz = *mut T;
            }
            impl<T> Bar for *const T {
                type Baz = *const T;
            }
        }
    );
}

#[test]
fn test_reference_types() {
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
        }
    );
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
        }
    );
    reparse_test!(
        program {
            trait Bar {
                type Baz;
            }
            impl<'a,T> Bar for &'a mut T {
                type Baz = &'a mut T;
            }
            impl<'a,T> Bar for &'a T {
                type Baz = &'a T;
            }
        }
    );
}

#[test]
fn test_tuples() {
    reparse_test!(
        program {
            struct Fuu {
                fuu_field: ()
            }
        }
    );
    reparse_test!(
        program {
            struct Uff {
                fuu_field: (Iff,),
                iff2_field: (Iff, Iff, Iff)
            }
            struct Iff { }
        }
    );
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
