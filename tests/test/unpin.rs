//! Tests targeting the Unpin trait

use super::*;

#[test]
fn unpin_lowering() {
    lowering_success! {
        program {
            #[auto] #[lang(unpin)] trait Unpin { }
            enum A { Variant }
            struct B { }
            impl !Unpin for A {}
            impl Unpin for B {}
        }
    }
}

#[test]
fn unpin_auto_trait() {
    test! {
        program {
            #[auto] #[lang(unpin)] trait Unpin { }
            struct A { }
        }

        goal {
            A: Unpin
        } yields {
            "Unique"
        }
    }
}

#[test]
fn unpin_negative() {
    test! {
        program {
            #[auto] #[lang(unpin)] trait Unpin { }
            struct A { }
            impl !Unpin for A {}
        }

        goal {
            A: Unpin
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn unpin_inherit_negative() {
    test! {
        program {
            #[auto] #[lang(unpin)] trait Unpin { }
            struct A { }
            impl !Unpin for A {}
            struct B { a: A }
        }

        goal {
            B: Unpin
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn unpin_overwrite() {
    test! {
        program {
            #[auto] #[lang(unpin)] trait Unpin { }
            struct A { }
            impl !Unpin for A {}
            struct B { a: A }
            impl Unpin for B {}
        }

        goal {
            B: Unpin
        } yields {
            "Unique"
        }
    }
}
