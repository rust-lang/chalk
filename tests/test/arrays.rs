use super::*;

#[test]
fn arrays_are_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            forall<const N> {
                [u32; N]: Sized
            }
        } yields {
            "Unique"
        }

    }
}

#[test]
fn arrays_are_copy_if_element_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }

            struct Foo { }
            impl Copy for Foo { }
        }

        goal {
            forall<const N> {
                [Foo; N]: Copy
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn arrays_are_not_copy_if_element_not_copy() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }

            struct Foo { }
        }

        goal {
            forall<const N> {
                [Foo; N]: Copy
            }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn arrays_are_clone_if_element_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            struct Foo { }
            impl Clone for Foo { }
        }

        goal {
            forall<const N> {
                [Foo; N]: Clone
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn arrays_are_not_clone_if_element_not_clone() {
    test! {
        program {
            #[lang(clone)]
            trait Clone { }

            struct Foo { }
        }

        goal {
            forall<const N> {
                [Foo; N]: Clone
            }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn arrays_are_well_formed_if_elem_sized() {
    test! {
        program {
            #[lang(sized)]
            trait Sized { }
        }

        goal {
            forall<const N, T> {
                if (T: Sized) {
                    WellFormed([T; N])
                }
            }
        } yields {
            "Unique"
        }

        goal {
            forall<const N, T> {
                WellFormed([T; N])
            }
        } yields {
            "No possible solution"
        }

        goal {
            exists<const N, T> {
                WellFormed([T; N])
            }
        } yields {
            "Ambiguous; no inference guidance"
        }
    }
}
