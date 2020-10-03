//! Tests for the static lifetime

use super::*;

#[test]
fn static_lowering() {
    lowering_success! {
        program {
            struct A<'a> where 'a: 'static {}
            trait B<'a> where 'a: 'static {}
            fn foo(a: &'static ());
        }
    }
}

#[test]
fn static_outlives() {
    test! {
        program {
            trait Foo<'a> where 'a: 'static {}
            struct Bar {}

            impl<'a> Foo<'a> for Bar where 'a: 'static {}
        }

        goal {
            exists<'a> {
                Bar: Foo<'a>
            }
        } yields {
            "Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: 'static }] }"
        }

        goal {
            forall<'a> {
                Bar: Foo<'a>
            }
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: 'static }]"
        }
    }
}

#[test]
fn static_impls() {
    test! {
        program {
            struct Foo {}
            trait Bar {}
            impl<'a> Bar for &'a Foo {}
        }

        goal {
            &'static Foo: Bar
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'a> { &'a Foo: Bar }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            exists<'a> { &'a Foo: Bar }
        } yields {
            "Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [] }"
        }
    }
}
