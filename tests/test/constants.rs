//! Tests related to const generics.

use super::*;

#[test]
fn single_impl() {
    test! {
        program {
            struct S<const N> {}

            trait Trait {}

            impl Trait for S<3> {}
        }

        goal {
            exists<const N> {
                S<N>: Trait
            }
        } yields {
            "Unique; substitution [?0 := 3], lifetime constraints []"
        }

        goal {
            S<3>: Trait
        } yields {
            "Unique"
        }

        goal {
            S<3?>: Trait
        } yields {
            "Unique"
        }

        goal {
            S<5>: Trait
        } yields {
            "No possible solution"
        }

        goal {
            S<5?>: Trait
        } yields {
            "No possible solution"
        }

        goal {
            S<?>: Trait
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            "No possible solution"
        }

    }
}

#[test]
fn multi_impl() {
    test! {
        program {
            struct S<const N> {}

            trait Trait {}

            impl Trait for S<3> {}
            impl Trait for S<5> {}
        }

        goal {
            exists<const N> {
                S<N>: Trait
            }
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            "No possible solution"
        }

    }
}

#[test]
fn generic_impl() {
    test! {
        program {
            struct S<const N> {}

            trait Trait {}

            impl<const N> Trait for S<N> {}
        }

        goal {
            exists<const N> {
                S<N>: Trait
            }
        } yields {
            "Unique; for<?U0> { substitution [?0 := ^0.0], lifetime constraints [] }"
        }

        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn unevaluated_impl() {
    test! {
        program {
            struct S<const N> {}

            trait Trait {}

            impl Trait for S<3?> {}
        }

        // Tests unification of unevaluated const and inference var
        goal {
            exists<const N> {
                S<N>: Trait
            }
        } yields {
            "Unique; substitution [?0 := 3?], lifetime constraints []"
        }

        // Tests unification of unevaluated const and placeholder const
        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            "No possible solution"
        }

        // Tests mismatched unification of unevaluated const and concrete const
        goal {
            S<8>: Trait
        } yields {
            "No possible solution"
        }

        // Tests matched unification of unevaluated const and concrete const
        goal {
            S<3>: Trait
        } yields {
            "Unique"
        }

        // Tests unification of known unevaluated const and unknown unevaluated const
        goal {
            S<?>: Trait
        } yields {
            "Ambiguous; no inference guidance"
        }

        // Tests matched unification of unevaluated consts
        goal {
            S<3?>: Trait
        } yields {
            "Unique"
        }

        // Tests mismatched unification of unevaluated consts
        goal {
            S<8?>: Trait
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn unevaluated_impl_generic() {
    test! {
        // FIXME: Coherence fails due to structural mismatch of ? and ?
        disable_coherence;
        program {
            struct S<const N> {}

            trait Trait {}

            impl Trait for S<?> {}
        }

        // Tests unification of unknown unevaluated const and evaluated const
        goal {
            S<0>: Trait
        } yields {
            "Ambiguous; no inference guidance"
        }

        // Tests unification of two unknown unevaluated consts
        goal {
            S<?>: Trait
        } yields {
            "Ambiguous; no inference guidance"
        }

        // Tests unification of unknown unevaluated const and inference var
        // We don't know if ? evaluates to a valid const, so even this is ambiguous
        goal {
            exists<const N> {
                S<N>: Trait
            }
        } yields {
            "Ambiguous; no inference guidance"
        }

        // Tests unification of unknown unevaluated const and placeholder const
        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn placeholders_eq() {
    test! {
        program {}

        goal {
            forall<const C, const D> {
                C = D
            }
        } yields {
            "No possible solution"
        }

        // TODO: Make this a const test
        goal {
            exists<C> {
                forall<D> {
                    C = D
                }
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<const C> {
                exists<const D> {
                    C = D
                }
            }
        } yields {
            "Unique; substitution [?0 := !1_0], lifetime constraints []"
        }

        goal {
            forall<const C1, const C2> {
                exists<const D1, const D2> {
                    C1 = D1, C2 = D2, D1 = D2
                }
            }
        } yields {
            "No possible solution"
        }
    }
}
