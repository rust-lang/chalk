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
            #[variance(Covariant)]
            struct Foo<'a> { }
        }

        goal {
            forall<'a, 'b> {
                Subtype(Foo<'a>, Foo<'b>)
            }
        } yields {
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 } \
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
            InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }\
            ]"
        }
    }
}

#[test]
fn fn_lifetime_variance_args() {
    test! {
        program {
        }

        goal {
            for<'a, 'b> fn(&'a u32, &'b u32) = for<'a> fn(&'a u32, &'a u32)
        } yields {
            "Unique;substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"
        }
    }
}

#[test]
fn fn_lifetime_variance_with_return_type() {
    test! {
        program {
        }

        goal {
            Subtype(for<'a, 'b> fn(&'a u32, &'b u32) -> &'a u32, for<'a> fn(&'a u32, &'a u32) -> &'a u32)
        } yields {
            // TODO: are these results actually correct?
            "Unique; for<?U1,?U1> { substitution [], lifetime constraints [\
                InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
                InEnvironment { environment: Env([]), goal: '!1_0: '^0.1 }, \
                InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }  \
            ]}"
        }
        goal {
            Subtype(for<'a> fn(&'a u32, &'a u32) -> &'a u32, for<'a, 'b> fn(&'a u32, &'b u32) -> &'a u32)
        } yields {
            "Unique; for<?U1> { substitution [], lifetime constraints [\
                InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, \
                InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }, \
                InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }  \
            ] }"
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
            "Unique; substitution [?0 := (&'!1_0 Uint(U32))], lifetime constraints []"
        }
    }
}

/// Tests that the generalizer correctly generalizes lifetimes (as opposed to
/// just that variance in general works).
#[test]
fn multi_lifetime() {
    test! {
        program {}

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(&'a u32, U),
                    Subtype(&'b u32, U)
                }
            }
        } yields {
            // Without the generalizer, we would yield a result like this:
            //
            // "Unique; substitution [?0 := (&'!1_1 Uint(U32))], lifetime
            // constraints [InEnvironment { environment: Env([]), goal: '!1_1: '!1_0
            // }]"
            //
            // This is incorrect, as we shouldn't be requiring 'a and 'b to be
            // related to eachother. Instead, U should be &'?1 u32, with constraints
            // ?1 : 'a, ?1: 'b.
            "Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.0  }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '^0.0  }] }"
        }
    }
}

/// Tests that the generalizer generalizes lifetimes within a covariant struct.
#[test]
fn multi_lifetime_covariant_struct() {
    test! {
        program {
            #[variance(Covariant)]
            struct Foo<A> {}
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<&'a u32>, Foo<U>),
                    Subtype(Foo<&'b u32>, Foo<U>)
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            "Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.0  }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '^0.0  }] }"
        }
    }
}

/// Tests that the generalizer generalizes lifetimes within a contravariant
/// struct, and that that variance contravariance is recognized.
#[test]
fn multi_lifetime_contravariant_struct() {
    test! {
        program {
            #[variance(Contravariant)]
            struct Foo<A> {}
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<&'a u32>, Foo<U>),
                    Subtype(Foo<&'b u32>, Foo<U>)
                }
            }
        } yields {
            // Result should be opposite multi_lifetime result.
            "Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, \
            InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"
        }
    }
}

/// Tests that the generalizer generalizes lifetimes within a invariant struct.
#[test]
fn multi_lifetime_invariant_struct() {
    test! {
        program {
            #[variance(Invariant)]
            struct Foo<A> {}
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<&'a u32>, Foo<U>),
                    Subtype(Foo<&'b u32>, Foo<U>)
                }
            }
        } yields {
            // Because A is invariant, we require the lifetimes to be equal
            "Unique; substitution [?0 := (&'!1_1 Uint(U32))], lifetime
            constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"
        }
    }
}

/// Tests that the generalizer generalizes lifetimes within slices correctly.
#[test]
fn multi_lifetime_slice() {
    test! {
        program {
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([&'a u32], [U]),
                    Subtype([&'b u32], [U])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            "Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.0  }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '^0.0  }] }"
        }
    }
}

/// Tests that the generalizer generalizes lifetimes within tuples correctly.
#[test]
fn multi_lifetime_tuple() {
    test! {
        program {
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype((&'a u32, &'a u32), [U]),
                    Subtype((&'b u32, &'b u32), [U])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            "Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.0  }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '^0.0  }] }"
        }
    }
}

/// Tests that the generalizer generalizes lifetimes within tuples correctly.
#[test]
fn multi_lifetime_array() {
    test! {
        program {
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([&'a u32; 16], [U]),
                    Subtype([&'b u32; 16], [U])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            "Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '^0.0  }, \
            InEnvironment { environment: Env([]), goal: '!1_1: '^0.0  }] }"
        }
    }
}
