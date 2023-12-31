use super::*;

// Test that user-provided impls of `Discriminantkind` are prohibited
#[test]
fn no_discriminant_kind_impls() {
    lowering_error! {
        program {
            #[lang(discriminant_kind)]
            trait DiscriminantKind {
                type Discriminant;
            }

            impl DiscriminantKind for u32 {
                type Discriminant = u32;
            }
        } error_msg {
            "trait impl for `DiscriminantKind` does not meet well-formedness requirements"
        }
    }
}

// Test that all types are implementing DiscriminantKind
#[test]
fn discriminant_kind_impl() {
    test! {
        program {
            #[lang(discriminant_kind)]
            trait DiscriminantKind {
                type Discriminant;
            }

            #[object_safe]
            trait Principal {}

            struct A { }
        }

        goal {
            A: DiscriminantKind
        } yields {
            expect![["Unique"]]
        }

        goal {
            i32: DiscriminantKind
        } yields {
            expect![["Unique"]]
        }

        goal {
            (i32, A): DiscriminantKind
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'a> {
                dyn Principal + 'a: DiscriminantKind
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn discriminant_kind_assoc() {
    test! {
        program {
            #[lang(discriminant_kind)]
            trait DiscriminantKind {
                type Discriminant;
            }

            #[object_safe]
            trait Principal {}

            enum A { }

            #[repr(isize)]
            enum B { }

            #[repr(i32)]
            enum C {}

            #[repr(u32)]
            enum D {}

            #[repr(usize)]
            enum E {}

            coroutine empty_gen<>[resume = (), yield = ()] {
                upvars []
                witnesses []
            }
        }

        // Discriminant for types with no discriminant should be u8
        goal {
            Normalize(<u32 as DiscriminantKind>::Discriminant -> u8)
        } yields {
            expect![["Unique"]]
        }

        // Same as above
        goal {
            forall<'a> {
                Normalize(<dyn Principal + 'a as DiscriminantKind>::Discriminant -> u8)
            }
        } yields {
            expect![["Unique"]]
        }

        // Discriminant for enums with unspecified discriminant should be isize
        goal {
            Normalize(<A as DiscriminantKind>::Discriminant -> isize)
        } yields {
            expect![["Unique"]]
        }

        // Discriminant should be the same as specified in `repr`
        // -----
        goal {
            Normalize(<B as DiscriminantKind>::Discriminant -> isize)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<C as DiscriminantKind>::Discriminant -> i32)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<D as DiscriminantKind>::Discriminant -> u32)
        } yields {
            expect![["Unique"]]
        }

        goal {
            Normalize(<E as DiscriminantKind>::Discriminant -> usize)
        } yields {
            expect![["Unique"]]
        }
        //--------

        // Coroutines have u32 as the discriminant
        goal {
            Normalize(<empty_gen as DiscriminantKind>::Discriminant -> u32)
        } yields {
            expect![["Unique"]]
        }

        // Placeholders don't have a determined discriminant
        goal {
            forall<T> {
                exists<U> {
                    <T as DiscriminantKind>::Discriminant = U
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := (DiscriminantKind::Discriminant)<!1_0>]"]]
        }
    }
}

#[test]
fn discriminant_kind_with_infer_var() {
    test! {
        program {
            #[lang(discriminant_kind)]
            trait DiscriminantKind {
                type Discriminant;
            }

            enum Option<T> {}
        }

        goal {
            exists<T> {
                Normalize(<Option<T> as DiscriminantKind>::Discriminant -> isize)
            }
        } yields {
            expect![[r#"Unique; for<?U0> { substitution [?0 := ^0.0] }"#]]
        }
    }
}
