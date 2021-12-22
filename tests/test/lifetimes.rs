//! Tests for various concrete lifetimes

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
fn erased_lowering() {
    lowering_success! {
        program {
            struct A<'a> where 'a: 'erased {}
            trait B<'a> where 'a: 'erased {}
            fn foo(a: &'erased ());
        }
    }
}

#[test]
fn empty_lowering() {
    lowering_success! {
        program {
            struct A<'a> where 'a: 'empty {}
            trait B<'a> where 'a: 'empty {}
            fn foo(a: &'empty ());
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
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: 'static }] }"]]
        }

        goal {
            forall<'a> {
                Bar: Foo<'a>
            }
        } yields {
            expect![["Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: 'static }]"]]
        }
    }
}

#[test]
fn empty_outlives() {
    test! {
        program {
            trait Foo<'a> where 'a: 'empty {}
            struct Bar {}

            impl<'a> Foo<'a> for Bar where 'a: 'empty {}
        }

        goal {
            exists<'a> {
                Bar: Foo<'a>
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '<empty> }] }"]]
        }

        goal {
            forall<'a> {
                Bar: Foo<'a>
            }
        } yields {
            expect![["Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '<empty> }]"]]
        }
    }
}

#[test]
fn erased_outlives() {
    test! {
        program {
            trait Foo<'a> where 'a: 'erased {}
            struct Bar {}

            impl<'a> Foo<'a> for Bar where 'a: 'erased {}
        }

        goal {
            exists<'a> {
                Bar: Foo<'a>
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [InEnvironment { environment: Env([]), goal: '^0.0: '<erased> }] }"]]
        }

        goal {
            forall<'a> {
                Bar: Foo<'a>
            }
        } yields {
            expect![["Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '<erased> }]"]]
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
            expect![["Unique; substitution [], lifetime constraints []"]]
        }

        goal {
            forall<'a> { &'a Foo: Bar }
        } yields {
            expect![["Unique; substitution [], lifetime constraints []"]]
        }

        goal {
            exists<'a> { &'a Foo: Bar }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0], lifetime constraints [] }"]]
        }
    }
}

#[test]
fn erased_impls() {
    test! {
        program {
            struct Foo {}
            trait Bar {}
            impl<'a> Bar for &'a Foo {}
        }

        goal {
            &'erased Foo: Bar
        } yields {
            expect![["Unique; substitution [], lifetime constraints []"]]
        }
    }
}

#[test]
fn empty_impls() {
    test! {
        program {
            struct Foo {}
            trait Bar {}
            impl<'a> Bar for &'a Foo {}
        }

        goal {
            &'empty Foo: Bar
        } yields {
            expect![["Unique; substitution [], lifetime constraints []"]]
        }
    }
}
