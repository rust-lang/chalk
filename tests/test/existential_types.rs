//! Tests related to the implied bounds rules.

use super::*;

#[test]
fn opaque_bounds() {
    test! {
        program {
            trait Direct { }
            trait Indirect { }
            struct Ty { }
            impl Direct for Ty { }
            impl Indirect for Ty { }

            opaque type T: Direct = Ty;
        }

        goal {
            T: Direct
        } yields {
            "Unique; substitution []"
        }

        goal {
            if (Reveal) {
                T: Indirect
            }
        } yields {
            "Unique; substitution []"
        }

        goal {
            T: Indirect
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn dyn_Clone_is_Clone() {
    test! {
        program {
            trait Clone { }
        }

        goal {
            dyn Clone: Clone
        } yields {
            "Unique; substitution []"
        }
    }
}

#[test]
fn dyn_Clone_is_not_Send() {
    test! {
        program {
            trait Clone { }
            #[auto] trait Send { }
        }

        goal {
            dyn Clone: Send
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn dyn_Clone_Send_is_Send() {
    test! {
        program {
            trait Clone { }
            #[auto] trait Send { }
        }

        goal {
            (dyn Clone + Send): Send
        } yields {
            "Unique; substitution []"
        }
    }
}

#[test]
fn dyn_Foo_Bar() {
    test! {
        program {
            trait Foo<T> { }

            struct Bar { }
            struct Baz { }
        }

        goal {
            dyn Foo<Bar>: Foo<Baz>
        } yields {
            "No possible solution"
        }

        goal {
            exists<T> {
                dyn Foo<T>: Foo<Bar>
            }
        } yields {
            "Unique; substitution [?0 := Bar], lifetime constraints []"
        }
    }
}

#[test]
fn dyn_higher_ranked_type_arguments() {
    test! {
        program {
            trait Foo<T> { }
            trait Bar { }

            struct Ref<'a> { }
        }

        goal {
            forall<'static> {
                dyn forall<'a> Foo<Ref<'a>>: Foo<Ref<'static>>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'static> {
                dyn forall<'a> Foo<Ref<'a>> + Bar: Foo<Ref<'static>>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            dyn forall<'a> Foo<Ref<'a>> + Bar: Bar
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'static> {
                forall<'a> {
                    dyn Foo<Ref<'static>>: Foo<Ref<'a>>
                }
            }
        } yields {
            // Note that this requires 'a == 'static, so it would be resolveable later on.
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_0 == '!1_0 }]"
        }
    }
}

#[test]
fn dyn_binders_reverse() {
    test! {
        program {
            trait Fn<T> { }

            trait Eq<A> { }

            struct Refs<'a, 'b> { }

            impl<A> Eq<A> for A { }
        }

        // Note: these constraints are ultimately unresolveable (we
        // have to show that 'a == 'b, basically)
        goal {
            dyn forall<'a, 'b> Fn<Refs<'a, 'b>>: Eq<
                dyn forall<'c> Fn<Refs<'c, 'c>>
            >
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!3_0 == '!3_1 }, InEnvironment { environment: Env([]), goal: '!6_0 == '!6_1 }]"
        }

        // Note: these constraints are ultimately unresolveable (we
        // have to show that 'a == 'b, basically)
        goal {
            dyn forall<'c> Fn<Refs<'c, 'c>>: Eq<
                dyn forall<'a, 'b> Fn<Refs<'a, 'b>>
            >
        } yields {
            "Unique; substitution [], lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_1 == '!2_0 }, InEnvironment { environment: Env([]), goal: '!5_1 == '!5_0 }]"
        }

        // Note: ordering of parameters is reversed here, but that's no problem
        goal {
            dyn forall<'c, 'd> Fn<Refs<'d, 'c>>: Eq<
                dyn forall<'a, 'b> Fn<Refs<'a, 'b>>
            >
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
