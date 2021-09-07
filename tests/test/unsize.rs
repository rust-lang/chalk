use crate::test::*;

#[test]
fn dyn_to_dyn_unsizing() {
    test! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            #[object_safe]
            trait Principal {}
            #[object_safe]
            trait OtherPrincipal {}
            #[object_safe]
            trait GenericPrincipal<T> {
                type Item;
            }

            #[auto]
            #[object_safe]
            trait Auto1 {}

            #[auto]
            #[object_safe]
            trait Auto2 {}

            #[auto]
            #[object_safe]
            trait Auto3 {}
        }

        // Tests with the same principal and auto traits
        goal {
            forall<'a> {
                forall<'b> {
                    dyn Principal + 'a: Unsize<dyn Principal + 'b>
                }
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }]"]]
        }

        goal {
            forall<'a> {
                forall<'b> {
                    dyn Principal + Auto1 + Auto2 + Auto3 + 'a: Unsize<dyn Principal + Auto1 + Auto2 + Auto3 + 'b>
                }
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }]"]]
        }

        // Target has a subset of source auto traits
        goal {
            forall<'a> {
                dyn Principal + Auto1 + Auto2 + 'a: Unsize<dyn Principal + Auto1 + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_0 }]"]]
        }

        // Both target and source don't have principal as their first trait
        goal {
            forall<'a> {
                dyn Auto1 + Principal + 'a: Unsize<dyn Auto1 + Principal + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_0 }]"]]
        }

        // Different order of traits in target and source
        goal {
            forall<'a> {
                dyn Principal + Auto1 + 'a: Unsize<dyn Auto1 + Principal + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_0 }]"]]
        }

        // See above
        goal {
            forall<'a> {
                dyn Principal + Auto2 + Auto1 + 'a: Unsize<dyn Principal + Auto1 + Auto2 + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_0 }]"]]
        }

        // Source has a subset of auto traits of target
        goal {
            forall<'a> {
                dyn Principal + Auto2 + 'a: Unsize<dyn Principal + Auto1 + Auto2 + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Source and target have different set of auto traits
        goal {
            forall<'a> {
                dyn Principal + Auto1 + Auto2 + 'a: Unsize<dyn Principal + Auto1 + Auto3 + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Source has a principal trait, while target doesnt, both have the same auto traits.
        goal {
            forall<'a> {
                dyn Principal + Auto1 + 'a: Unsize<dyn Auto1 + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Non-matching principal traits
        goal {
            forall<'a> {
                dyn Principal + 'a: Unsize<dyn OtherPrincipal + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Matching generic principal traits
        goal {
            forall<'a> {
                dyn GenericPrincipal<u64, Item = u64> + 'a: Unsize<dyn GenericPrincipal<u64, Item = u64> + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!1_0 }]"]]
        }

        // Non-matching generic principal traits
        goal {
            forall<'a> {
                dyn GenericPrincipal<u32, Item = u32> + 'a: Unsize<dyn GenericPrincipal<u32, Item = u64> + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn ty_to_dyn_unsizing() {
    test! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}
            #[lang(sized)]
            trait Sized {}

            #[object_safe]
            trait Principal {}
            #[object_safe]
            trait GenericPrincipal<T> {
                type Item;
            }

            trait UnsafePrincipal {}

            #[auto]
            #[object_safe]
            trait Auto {}

            struct Foo {}
            struct FooLifetime<'a> {}
            struct Bar {}
            struct Baz {}
            struct FooNotSized<T> {
                t: T
            }

            impl Principal for Foo {}
            impl UnsafePrincipal for Foo {}

            impl<'a> Principal for FooLifetime<'a> {}

            impl Principal for Bar {}
            impl !Auto for Bar {}

            impl<T> Principal for FooNotSized<T> {}

            impl GenericPrincipal<u32> for Foo {
                type Item = u32;
            }
        }

        goal {
            forall<'a> {
                Foo: Unsize<dyn Principal + Auto + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // Principal is not the first trait
        goal {
            forall<'a> {
                Foo: Unsize<dyn Auto + Principal + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // Auto-only trait object
        goal {
            forall<'a> {
                Foo: Unsize<dyn Auto + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // TypeOutlives test
        goal {
            forall<'a> {
                FooLifetime<'a>: Unsize<dyn Principal + Auto + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: FooLifetime<'!1_0>: '!1_0 }]"]]
        }

        // See above
        goal {
            forall<'a> {
                exists<'b> {
                    FooLifetime<'a>: Unsize<dyn Principal + Auto + 'b>
                }
            }
        } yields {
            expect![["Unique; for<?U1> { substitution [?0 := '^0.0], lifetime constraints [InEnvironment { environment: Env([]), goal: FooLifetime<'!1_0>: '^0.0 }] }"]]
        }

        // Source does not implement auto trait (with principal)
        goal {
            forall<'a> {
                Bar: Unsize<dyn Principal + Auto + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Source does not implement auto trait (without principal)
        goal {
            forall<'a> {
                Bar: Unsize<dyn Auto + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Source does not implement principal
        goal {
            forall<'a> {
                Baz: Unsize<dyn Principal + Auto + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Implemeted generic principal
        goal {
            forall<'a> {
                Foo: Unsize<dyn GenericPrincipal<u32, Item = u32> + 'a>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }


        // Non-implemeted generic principal
        goal {
            forall<'a> {
                Foo: Unsize<dyn GenericPrincipal<u32, Item = u64> + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Not object-safe principal trait
        goal {
            forall<'a> {
                Foo: Unsize<dyn UnsafePrincipal + 'a>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Source ty is not Sized
        goal {
            forall<'a> {
                forall<T> {
                    FooNotSized<T>: Unsize<dyn Principal + 'a>
                }
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Sized counterpart for the previous test
        goal {
            forall<'a> {
                forall<T> {
                    if (T: Sized) {
                       FooNotSized<T>: Unsize<dyn Principal + 'a>
                    }
                }
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: FooNotSized<!2_0>: '!1_0 }]"]]
        }
    }
}

#[test]
fn tuple_unsizing() {
    test! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}
            #[lang(sized)]
            trait Sized {}

            struct Foo {}

            #[object_safe]
            trait Principal {}
            #[object_safe]
            trait OtherPrincipal {}

            impl Principal for Foo {}
        }

        goal {
            (): Unsize<()>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            (u32, u32): Unsize<(u32, u32)>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<'a> {
               (u32, Foo): Unsize<(u32, dyn Principal + 'a)>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // Last field does not implement `Unsize`
        goal {
            forall<'a> {
               (u32, Foo): Unsize<(u32, dyn OtherPrincipal + 'a)>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Mismatch of head fields
        goal {
            forall<'a> {
               (u32, Foo): Unsize<(u64, dyn Principal + 'a)>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Tuple length mismatch
        goal {
            forall<'a> {
               (u32, u32, Foo): Unsize<(u32, dyn Principal + 'a)>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Multilevel tuple test
        goal {
            forall<'a> {
               (u32, (u32, Foo)): Unsize<(u32, (u32, dyn Principal + 'a))>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }
    }
}

#[test]
fn array_unsizing() {
    test! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}

            struct Foo<T> {}
        }

        goal {
            [Foo<u8>; 2]: Unsize<[Foo<u8>]>
        } yields {
            expect![["Unique"]]
        }

        goal {
            [Foo<u8>; 5]: Unsize<[Foo<u16>]>
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn struct_unsizing() {
    test! {
        program {
            #[lang(unsize)]
            trait Unsize<T> {}
            #[lang(sized)]
            trait Sized {}

            struct Foo {}
            struct Bar<T> {}
            struct Baz<'a> {}

            struct S1<T1> {
                t1: T1
            }

            struct S12<T1, T2> where T1: Sized {
                t1: T1,
                t2: T2
            }

            struct SParamsInMultipleFields<T> {
                t1: Bar<T>,
                t2: T
            }

            struct SNested<T1, T2, T3> where T1: Sized, T2: Sized {
                t1: T1,
                t2: (T2, S1<T3>)
            }

            struct SBad<T1, T2> where T1: Sized {
                t1: Bar<S1<T2>>,
                t2: (T1, S1<T2>)
            }

            struct SWithBinders<T3, T1> where T1: Sized {
                t1: T1,
                t2: for<'a> fn(dyn Principal + 'a),
                t3: T3
            }

            struct SLifetime<'a, T> {
                t1: Baz<'a>,
                t2: S12<Baz<'a>, T>
            }

            struct SConst<const N, T> {
                t: T
            }

            struct SGoodConst<const N, T> {
                t1: u32,
                t2: SConst<N, T>
            }

            struct SBadConst<const N, T> {
                t1: [u32; N],
                t2: SConst<N, T>
            }

            #[object_safe]
            trait Principal {}
            #[object_safe]
            trait OtherPrincipal {}

            impl Principal for Foo {}
        }

        // Single field struct tests
        goal {
            Foo: Unsize<Foo>
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<'a> {
                S1<Foo>: Unsize<S1<dyn Principal + 'a>>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        goal {
            forall<'a> {
                S1<Foo>: Unsize<S1<dyn OtherPrincipal + 'a>>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Unsizing parameter is used in head fields
        goal {
            forall<'a> {
                SParamsInMultipleFields<Foo>:
                    Unsize<SParamsInMultipleFields<dyn Principal + 'a>>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Two-field struct test
        goal {
            forall<'a> {
                S12<Foo, Foo>: Unsize<S12<Foo, dyn Principal + 'a>>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // Test for the unsizing parameters collector
        // (checking that it ignores the binder inside `SWithBinders`)
        goal {
            forall<'a> {
                SWithBinders<Foo, Foo>: Unsize<SWithBinders<dyn Principal + 'a, Foo>>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // Non-trivial unsizing of the last field
        goal {
            forall<'a> {
                SNested<Foo, Bar<Foo>, Foo>: Unsize<SNested<Foo, Bar<Foo>, dyn Principal + 'a>>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        goal {
            forall<'a> {
                SBad<Foo, Foo>: Unsize<SBad<Foo, dyn Principal + 'a>>
            }
        } yields {
            expect![["No possible solution"]]
        }

        // Check that lifetimes can't be used as unsizing parameters
        goal {
            forall<'a> {
                SLifetime<'a, Foo>: Unsize<SLifetime<'a, dyn Principal + 'a>>
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: Foo: '!1_0 }]"]]
        }

        // Tests with constant as an unsizing parameter
        goal {
            SGoodConst<5, [u32; 2]>: Unsize<SGoodConst<5, [u32]>>
        } yields {
            expect![["Unique"]]
        }


        // Target does not match source
        goal {
            SGoodConst<4, [u32; 2]>: Unsize<SGoodConst<5, [u32]>>
        } yields {
            expect![["No possible solution"]]
        }

        // Unsizing parameter is used in head fields
        goal {
            SBadConst<5, [u32; 2]>: Unsize<SBadConst<5, [u32]>>
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn dispatch_from_dyn() {
    test! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            impl<'a, T, U> DispatchFromDyn<&'a U> for &'a T {}
        }

        // Smoke test that DispatchFromDyn works just like any other impl.
        goal {
            forall<'a> {
                &'a u8: DispatchFromDyn<&'a u8>
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn dispatch_from_dyn_wf() {
    lowering_success! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            #[one_zst]
            struct PhantomData<T> {}

            struct Foo<T> {
                f: *mut T,
                f2: PhantomData<u8>,
            }

            // References and pointers
            impl<'a, T, U> DispatchFromDyn<&'a U> for &'a T {}
            impl<'a, T, U> DispatchFromDyn<&'a mut U> for &'a mut T {}
            impl<T, U> DispatchFromDyn<*const U> for *const T {}
            impl<T, U> DispatchFromDyn<*mut U> for *mut T {}

            // Struct
            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        }
    }

    // Reference: mutability mismatch
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            impl<'a, T, U> DispatchFromDyn<&'a U> for &'a mut T {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // Raw pointer: mutability mismatch
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            impl<'a, T, U> DispatchFromDyn<*mut U> for *const T {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // No non-ZST fields
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            #[one_zst]
            struct PhantomData<T> {}

            struct Foo<T> {
                f: PhantomData<T>,
            }

            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // Too many fields
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            struct Foo<T> {
                f: *mut T,
                f2: u8,
            }

            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // Field does not impl DispatchFromDyn
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            struct Foo<T> {
                f: T,
            }

            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // Field type does not change
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            #[one_zst]
            struct PhantomData<T> {}

            struct Foo<T> {
                f: *const u8,
                f2: PhantomData<T>,
            }

            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // Different definitions
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            struct Foo<T> {
                f: *const T,
            }

            struct Bar<T> {
                f: *const T,
            }

            impl<T, U> DispatchFromDyn<Bar<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // Not a struct
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            enum Foo<T> {
                Bar(*const T),
            }

            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }

    // repr(C)
    lowering_error! {
        program {
            #[lang(dispatch_from_dyn)]
            trait DispatchFromDyn<T> {}

            #[repr(C)]
            struct Foo<T> {
                f: *mut T,
            }

            impl<T, U> DispatchFromDyn<Foo<U>> for Foo<T> {}
        } error_msg {
            "trait impl for `DispatchFromDyn` does not meet well-formedness requirements"
        }
    }
}
