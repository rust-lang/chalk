//! Tests for extern types

use super::*;

#[test]
fn extern_ty_is_well_formed() {
    test! {
        program {
            extern { type A; }
        }

        goal { WellFormed(A) } yields { "Unique" }
    }
}

#[test]
fn extern_ty_is_not_sized() {
    test! {
        program {
            extern { type A; }
        }

        goal { not { A: Sized } } yields { "Unique" }
    }
}

#[test]
fn extern_ty_is_not_copy() {
    test! {
        program {
            extern { type A; }
        }

        goal { not { A: Copy } } yields { "Unique" }
    }
}

#[test]
fn extern_ty_is_not_clone() {
    test! {
        program {
            extern { type A; }
        }

        goal { not { A: Clone } } yields { "Unique" }
    }
}
