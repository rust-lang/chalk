use crate::test::*;

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
            struct Zst<T> {}

            struct Foo<T> {
                f: *mut T,
                f2: Zst<u8>,
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
            struct Zst<T> {}

            struct Foo<T> {
                f: Zst<T>,
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
            struct Zst<T> {}

            struct Foo<T> {
                f: *const u8,
                f2: Zst<T>,
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
