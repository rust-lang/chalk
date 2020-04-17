//! Tests related to opaque types

use super::*;

#[test]
fn opaque_bounds() {
    test! {
        program {
            struct Ty { }

            trait Clone { }
            opaque type T: Clone = Ty;
        }

        goal {
            T: Clone
        } yields {
            "Unique; substitution []"
        }
    }
}

#[test]
fn opaque_reveal() {
    test! {
        program {
            struct Ty { }
            trait Trait { }
            impl Trait for Ty { }

            trait Clone { }
            opaque type T: Clone = Ty;
        }

        goal {
            if (Reveal) {
                T: Trait
            }
        } yields {
            "Unique; substitution []"
        }

        goal {
            T: Trait
        } yields {
            "No possible solution"
        }
    }
}
