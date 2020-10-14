use super::*;

#[test]
fn variance_lowering() {
    lowering_success! {
        program {
            #[variance(Invariant, Covariant)]
            struct Foo<T, U> {}
            struct Bar<T, U> {}
            #[variance(Invariant, Contravariant)]
            fn foo<T, U>(t: T, u: U);
            fn bar<T, U>(t: T, u: U);
        }
    }
}

#[test]
fn subtype_simple() {
    test! {
        program {
            struct Foo { }
        }

        goal {
            Subtype(Foo, Foo)
        } yields {
            "Unique"
        }
    }
}

/// Test that `Foo<'a>` and `Foo<'b>` can be subtypes
/// if we constrain the lifetimes `'a` and `'b` appropriately.
#[test]
fn struct_lifetime_variance() {
    test! {
        program {
            struct Foo<'a> { }
        }

        goal {
            forall<'a, 'b> {
                Subtype(Foo<'a>, Foo<'b>)
            }
        } yields {
            // FIXME: we should really just require this in one direction?
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }\
            ]"
        }
    }
}

/// Test that `&'a u32 <: &'b u32` if `'a: 'b`
#[test]
fn ref_lifetime_variance() {
    test! {
        program {
        }

        goal {
            forall<'a, 'b> {
                Subtype(&'a u32, &'b u32)
            }
        } yields {
            // Seems good!
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }\
            ]"
        }
    }
}

#[test]
fn fn_lifetime_variance() {
    test! {
        program {
        }

        goal {
            Subtype(for<'a, 'b> fn(&'a u32, &'b u32), for<'a> fn(&'a u32, &'a u32))
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
#[test]
fn generalize() {
    test! {
        program {
            struct Foo<T> { }
        }

        goal {
            forall<'a> {
                exists<U> {
                    Subtype(Foo<&'a u32>, Foo<U>)
                }
            }
        } yields {
            // If this is invariant, then the generalizer might be doing
            // the right thing here by creating the general form of `&'a u32` equal to
            // just `&'a u32`
            "Unique; substitution [?0 := Not<'!1_0, Uint(U32)>], lifetime constraints []"
        }
    }
}
