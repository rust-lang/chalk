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

        goal {
            exists<const N> {
                S<N>: Trait
            }
        } yields {
            "Unique; substitution [?0 := 3], lifetime constraints []"
        }

        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            "No possible solution"
        }

        goal {
            S<8>: Trait
        } yields {
            "No possible solution"
        }

        goal {
            S<3>: Trait
        } yields {
            "Unique"
        }

        goal {
            S<?>: Trait
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            S<3?>: Trait
        } yields {
            "Unique"
        }

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
        program {
            struct S<const N> {}

            trait Trait {}

            impl Trait for S<?> {}
        }

        goal {
            S<0>: Trait
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            S<?>: Trait
        } yields {
            "Unique"
        }

        // We don't know if the constant evaluates to an invalid value
        goal {
            exists<N> {
                S<N>: Trait
            }
        } yields {
            "Ambiguous; no inference guidance"
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
            forall<C> {
                exists<D> {
                    C = D
                }
            }
        } yields {
            "Unique; substitution [?0 := !1_0], lifetime constraints []"
        }

        goal {
            forall<C1, C2> {
                exists<D1, D2> {
                    C1 = D1, C2 = D2, D1 = D2
                }
            }
        } yields {
            "No possible solution"
        }
    }
}
