use super::*;

#[test]
fn str_trait_impl() {
    test! {
        program {
            trait Foo {}
            impl Foo for str {}
        }

        goal { str: Foo } yields { "Unique" }
    }
}

#[test]
fn str_is_well_formed() {
    test! {
        program {}
        goal { WellFormed(str) } yields { "Unique" }
    }
}

#[test]
fn str_is_not_sized() {
    test! {
        program {
            #[lang(sized)] trait Sized {}
        }

        goal { not { str: Sized } } yields { "Unique" }
    }
}

#[test]
fn str_is_not_copy() {
    test! {
        program {
            #[lang(copy)] trait Copy {}
        }

        goal { not { str: Copy } } yields { "Unique" }
    }
}

#[test]
fn str_is_not_clone() {
    test! {
        program {
            #[lang(clone)] trait Clone {}
        }

        goal { not { str: Clone } } yields { "Unique" }
    }
}
