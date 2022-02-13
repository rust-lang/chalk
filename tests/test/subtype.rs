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
            expect![["Unique"]]
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
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }
    }
}

/// Test that `&'a u32 <: &'b u32` if `'a: 'b`
#[test]
fn ref_lifetime_variance() {
    test! {
        goal {
            forall<'a, 'b> {
                Subtype(&'a u32, &'b u32)
            }
        } yields {
            // Seems good!
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }]"]]
        }
    }
}

#[test]
fn fn_lifetime_variance_args() {
    test! {
        goal {
            for<'a, 'b> fn(&'a u32, &'b u32) = for<'a> fn(&'a u32, &'a u32)
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; for<?U1,?U2,?U2> { lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }, InEnvironment { environment: Env([]), goal: '!2_0: '^0.1 }, InEnvironment { environment: Env([]), goal: '!2_0: '^0.2 }] }"]]
        } yields[SolverChoice::slg_default()] {
            expect![["Unique; for<?U2,?U2,?U1> { lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.2 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.2 }, InEnvironment { environment: Env([]), goal: '!2_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!2_0: '^0.1 }] }"]]
        }
    }
}

#[test]
fn fn_lifetime_variance_with_return_type() {
    test! {
        goal {
            Subtype(for<'a, 'b> fn(&'a u32, &'b u32) -> &'a u32, for<'a> fn(&'a u32, &'a u32) -> &'a u32)
        } yields {
            // TODO: are these results actually correct?
            expect![["Unique; for<?U1,?U1> { lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_0: '^0.1 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }] }"]]
        }
        goal {
            Subtype(for<'a> fn(&'a u32, &'a u32) -> &'a u32, for<'a, 'b> fn(&'a u32, &'b u32) -> &'a u32)
        } yields {
            expect![["Unique; for<?U1> { lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }] }"]]
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
            expect![["Unique; substitution [?0 := (&'!1_0 Uint(U32))]"]]
        }
    }
}

/// Tests that the generalizer correctly generalizes lifetimes.
#[test]
fn multi_lifetime() {
    test! {
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
            // expect![["Unique; substitution [?0 := (&'!1_1 Uint(U32))], lifetime
            // constraints [InEnvironment { environment: Env([]), goal: '!1_1: '!1_0
            // }]"]]
            //
            // This is incorrect, as we shouldn't be requiring 'a and 'b to be
            // related to eachother. Instead, U should be &'?1 u32, with constraints
            // ?1 : 'a, ?1: 'b.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
    }
}

/// Tests that the generalizer correctly generalizes lifetimes when given an
/// inference var on the left hand side.
#[test]
fn multi_lifetime_inverted() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(U, &'a u32),
                    Subtype(U, &'b u32)
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
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that we handle variance for covariant structs correctly.
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
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<U>, Foo<&'a u32>),
                    Subtype(Foo<U>, Foo<&'b u32>)
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that we handle variance for contravariant structs correctly.
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
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<U>, Foo<&'a u32>),
                    Subtype(Foo<U>, Foo<&'b u32>)
                }
            }
        } yields {
            // Result should be opposite multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
    }
}

/// Tests that we handle variance for invariant structs correctly.
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
        } yields[SolverChoice::recursive_default()] {
            // Because A is invariant, we require the lifetimes to be equal
            expect![["Unique; substitution [?0 := (&'!1_0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        } yields[SolverChoice::slg_default()] {
            // Because A is invariant, we require the lifetimes to be equal
            expect![["Unique; substitution [?0 := (&'!1_1 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<U>, Foo<&'a u32>),
                    Subtype(Foo<U>, Foo<&'b u32>)
                }
            }
        } yields[SolverChoice::recursive_default()] {
            // Because A is invariant, we require the lifetimes to be equal
            expect![["Unique; substitution [?0 := (&'!1_0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        } yields[SolverChoice::slg_default()] {
            // Because A is invariant, we require the lifetimes to be equal
            expect![["Unique; substitution [?0 := (&'!1_1 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }
    }
}

/// Tests that we handle variance for slices correctly.
#[test]
fn multi_lifetime_slice() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([&'a u32], [U]),
                    Subtype([&'b u32], [U])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([U], [&'a u32]),
                    Subtype([U], [&'b u32])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that we handle variance for tuples correctly.
#[test]
fn multi_lifetime_tuple() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype((&'a u32,), (U,)),
                    Subtype((&'b u32,), (U,))
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype((U,), (&'a u32,)),
                    Subtype((U,), (&'b u32,))
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that we handle variance for arrays correctly.
#[test]
fn multi_lifetime_array() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([&'a u32; 16], [U; 16]),
                    Subtype([&'b u32; 16], [U; 16])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([U; 16], [&'a u32; 16]),
                    Subtype([U; 16], [&'b u32; 16])
                }
            }
        } yields {
            // Result should be identical to multi_lifetime result.
            expect![["Unique; for<?U1> { substitution [?0 := (&'^0.0 Uint(U32))], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that the generalizer recurses into covariant structs correctly.
#[test]
fn generalize_covariant_struct() {
    test! {
        program {
            #[variance(Covariant)]
            struct Foo<A> {}
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<&'a u32>, U),
                    Subtype(Foo<&'b u32>, U)
                }
            }
        } yields {
            expect![["Unique; for<?U1> { substitution [?0 := Foo<(&'^0.0 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
    }
}

/// Tests that the generalizer recurses into contravariant structs correctly.
#[test]
fn generalize_contravariant_struct() {
    test! {
        program {
            #[variance(Contravariant)]
            struct Foo<A> {}
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<&'a u32>, U),
                    Subtype(Foo<&'b u32>, U)
                }
            }
        } yields {
            // Result should be opposite generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := Foo<(&'^0.0 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that the generalizer recurses into invariant structs correctly.
#[test]
fn generalize_invariant_struct() {
    test! {
        program {
            #[variance(Invariant)]
            struct Foo<A> {}
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(Foo<&'a u32>, U),
                    Subtype(Foo<&'b u32>, U)
                }
            }
        } yields[SolverChoice::recursive_default()] {
            // Because A is invariant, we require the lifetimes to be equal
            expect![["Unique; substitution [?0 := Foo<(&'!1_0 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        } yields[SolverChoice::slg_default()] {
            expect![["Unique; substitution [?0 := Foo<(&'!1_1 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_1 }, InEnvironment { environment: Env([]), goal: '!1_1: '!1_0 }]"]]
        }
    }
}

/// Tests that the generalizer recurses into slices correctly.
#[test]
fn generalize_slice() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([&'a u32], U),
                    Subtype([&'b u32], U)
                }
            }
        } yields {
            // Result should be identical to generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := [(&'^0.0 Uint(U32))]], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(U, [&'a u32]),
                    Subtype(U, [&'b u32])
                }
            }
        } yields {
            // Result should be identical to generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := [(&'^0.0 Uint(U32))]], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that the generalizer recurses into tuples correctly.
#[test]
fn generalize_tuple() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype((&'a u32,), U),
                    Subtype((&'b u32,), U)
                }
            }
        } yields {
            // Result should be identical to generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := 1<(&'^0.0 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(U, (&'a u32,)),
                    Subtype(U, (&'b u32,))
                }
            }
        } yields {
            // Result should be identical to generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := 1<(&'^0.0 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}

/// Tests that the generalizer recurses into N-tuples correctly.
#[test]
fn generalize_2tuple() {
    test! {
        goal {
            forall<'a, 'b, 'c, 'd> {
                exists<U> {
                    Subtype((&'a u32, &'c u32), U),
                    Subtype((&'b u32, &'d u32), U)
                }
            }
        } yields {
            expect![["Unique; for<?U1,?U1> { substitution [?0 := 2<(&'^0.0 Uint(U32)), (&'^0.1 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_2: '^0.1 }, InEnvironment { environment: Env([]), goal: '!1_3: '^0.1 }] }"]]
        }
        goal {
            forall<'a, 'b, 'c, 'd> {
                exists<U> {
                    Subtype(U, (&'a u32, &'c u32)),
                    Subtype(U, (&'b u32, &'d u32))
                }
            }
        } yields {
            expect![["Unique; for<?U1,?U1> { substitution [?0 := 2<(&'^0.0 Uint(U32)), (&'^0.1 Uint(U32))>], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }, InEnvironment { environment: Env([]), goal: '^0.1: '!1_2 }, InEnvironment { environment: Env([]), goal: '^0.1: '!1_3 }] }"]]
        }
    }
}

/// Tests that the generalizer recurses into arrays correctly.
#[test]
fn generalize_array() {
    test! {
        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype([&'a u32; 16], U),
                    Subtype([&'b u32; 16], U)
                }
            }
        } yields {
            // Result should be identical to generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := [(&'^0.0 Uint(U32)); 16]], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '^0.0 }, InEnvironment { environment: Env([]), goal: '!1_1: '^0.0 }] }"]]
        }

        goal {
            forall<'a, 'b> {
                exists<U> {
                    Subtype(U, [&'a u32; 16]),
                    Subtype(U, [&'b u32; 16])
                }
            }
        } yields {
            // Result should be identical to generalize_covariant_struct result.
            expect![["Unique; for<?U1> { substitution [?0 := [(&'^0.0 Uint(U32)); 16]], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '!1_0 }, InEnvironment { environment: Env([]), goal: '^0.0: '!1_1 }] }"]]
        }
    }
}
