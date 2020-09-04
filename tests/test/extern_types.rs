//! Tests for extern types

use super::*;

#[test]
fn extern_ty_trait_impl() {
    test! {
        program {
            extern { type A; }
            trait Foo {}
            impl Foo for A {}
        }

        goal { A: Foo } yields { "Unique" }
    }
}

#[test]
fn extern_ty_single() {
    lowering_success! {
        program {
            extern { type A; }
        }
    }
}

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
            #[lang(sized)] trait Sized {}
            extern { type A; }
        }

        goal { not { A: Sized } } yields { "Unique" }
    }
}

#[test]
fn extern_ty_is_not_copy() {
    test! {
        program {
            #[lang(copy)] trait Copy {}
            extern { type A; }
        }

        goal { not { A: Copy } } yields { "Unique" }
    }
}

#[test]
fn extern_ty_is_not_clone() {
    test! {
        program {
            #[lang(clone)] trait Clone {}
            extern { type A; }
        }

        goal { not { A: Clone } } yields { "Unique" }
    }
}
