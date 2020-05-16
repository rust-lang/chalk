use super::*;
#[test]
fn test_function_type() {
    reparse_test(
        "
            struct Foo { }
            trait Baz<T> { }
            impl Baz<fn(Foo)> for Foo { }
            ",
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
                ",
            scalar
        ));
    }
}

#[test]
fn test_str_types() {
    reparse_test(
        "
            struct Foo {
                field: str
            }
            trait Bar {
                type Baz;
            }
            impl Bar for Foo {
                type Baz = str;
            }
            ",
    );
}

#[test]
fn test_raw_ptr_types() {
    reparse_test(
        "
            struct Foo<T> {
                field: *const T
            }
            trait Bar {
                type Baz;
            }
            impl<T> Bar for Foo<T> {
                type Baz = *const u32;
            }
            ",
    );
    reparse_test(
        "
            struct Foo<T> {
                field: *mut T
            }
            trait Bar {
                type Baz;
            }
            impl<T> Bar for Foo<T> {
                type Baz = *mut u32;
            }
            ",
    );
}

#[test]
fn test_reference_types() {
    reparse_test(
        "
            struct Foo<'a,T> {
                field: &'a T
            }
            trait Bar {
                type Baz;
            }
            impl<'a,T> Bar for Foo<'a,T> {
                type Baz = &'a u32;
            }
            ",
    );
    reparse_test(
        "
            struct Foo<'a,T> {
                field: &'a mut T
            }
            trait Bar {
                type Baz;
            }
            impl<'a,T> Bar for Foo<'a,T> {
                type Baz = &'a mut u32;
            }
            ",
    );
}

#[test]
fn test_tuples() {
    reparse_test(
        "
            struct Fuu {
                fuu_field: ()
            }
            ",
    );
    reparse_test(
        "
            struct Uff {
                fuu_field: (Iff,),
                iff2_field: (Iff, Iff, Iff)
            }
            struct Iff { }
            ",
    );
    reparse_test(
        "
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
            ",
    );
}
