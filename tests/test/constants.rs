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
            expect![["Unique; substitution [?0 := 3]"]]
        }

        goal {
            S<3>: Trait
        } yields {
            expect![["Unique"]]
        }

        goal {
            S<5>: Trait
        } yields {
            expect![["No possible solution"]]
        }


        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            expect![["No possible solution"]]
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
            expect![["Ambiguous; no inference guidance"]]
        }

        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            expect![["No possible solution"]]
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
            expect![["Unique; for<?U0> { substitution [?0 := ^0.0] }"]]
        }

        goal {
            forall<const N> {
                S<N>: Trait
            }
        } yields {
            expect![["Unique"]]
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
            expect![["No possible solution"]]
        }

        goal {
            exists<const C> {
                forall<const D> {
                    C = D
                }
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<const C> {
                exists<const D> {
                    C = D
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := !1_0]"]]
        }

        goal {
            forall<const C1, const C2> {
                exists<const D1, const D2> {
                    C1 = D1, C2 = D2, D1 = D2
                }
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}
