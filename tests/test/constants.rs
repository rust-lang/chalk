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
            S<5>: Trait
        } yields {
            "No possible solution"
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
fn const_expr() {
    test! {
        const_eval {
            |name, args| {
                match name.as_str() {
                    "le128" => (args[0] < 128) as u32,
                    _ => panic!("Unknown function"),
                }
            }
        }

        program {
            struct S<const N> {}

            trait Trait {}

            struct Assert<const E> {}
            trait IsTrue {}
            impl IsTrue for Assert<1> {}

            impl<const N> Trait for S<N> where Assert<#le128(N)>: IsTrue {}
        }

        goal {
            S<3>: Trait
        } yields {
            "Unique; substitution[], lifetime constraints []"
        }

        goal {
            S<240>: Trait
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn placeholders_eq() {
    test! {
        goal {
            forall<const C, const D> {
                C = D
            }
        } yields {
            "No possible solution"
        }

        goal {
            exists<const C> {
                forall<const D> {
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
