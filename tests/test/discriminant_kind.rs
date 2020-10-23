use super::*;

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
            "Unique"
        }

        goal {
            i32: DiscriminantKind
        } yields {
            "Unique"
        }

        goal {
            (i32, A): DiscriminantKind
        } yields {
            "Unique"
        }

        goal {
            forall<'a> {
                dyn Principal + 'a: DiscriminantKind
            }
        } yields {
            "Unique"
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

            generator empty_gen<>[resume = (), yield = ()] {
                upvars []
                witnesses []
            }
        }

        goal {
            Normalize(<u32 as DiscriminantKind>::Discriminant -> u8)
        } yields {
            "Unique"
        }

        goal {
            forall<'a> {
                Normalize(<dyn Principal + 'a as DiscriminantKind>::Discriminant -> u8)
            }
        } yields {
            "Unique"
        }

        goal {
            Normalize(<A as DiscriminantKind>::Discriminant -> isize)
        } yields {
            "Unique"
        }

        goal {
            Normalize(<B as DiscriminantKind>::Discriminant -> isize)
        } yields {
            "Unique"
        }

        goal {
            Normalize(<C as DiscriminantKind>::Discriminant -> i32)
        } yields {
            "Unique"
        }

        goal {
            Normalize(<D as DiscriminantKind>::Discriminant -> u32)
        } yields {
            "Unique"
        }

        goal {
            Normalize(<E as DiscriminantKind>::Discriminant -> usize)
        } yields {
            "Unique"
        }

        goal {
            Normalize(<empty_gen as DiscriminantKind>::Discriminant -> u32)
        } yields {
            "Unique"
        }

        goal {
            forall<T> {
                exists<U> {
                    Normalize(<T as DiscriminantKind>::Discriminant -> U)
                }
            }
        } yields {
            "No possible solution"
        }
    }
}
