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

#[test]
fn opaque_generics() {
    test! {
        program {
            trait Iterator { type Item; }

            struct Vec<T> { }
            struct u32 { }
            impl<T> Iterator for Vec<T> {
                type Item = T;
            }

            opaque type Foo<X>: Iterator<Item = X> = Vec<X>;
        }

        goal {
            Foo<u32>: Iterator<Item = u32>
        } yields {
            "Unique; substitution []"
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    Foo<T>: Iterator<Item = T>
                }
            }
        } yields {
            "Unique; substitution []"
        }
    }
}
