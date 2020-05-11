//! Tests related to the implied bounds rules.

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
fn dyn_super_trait_simple() {
    test! {
        program {
            trait Foo<T> {}
            trait Bar<T> where Self: Foo<T> {}

            struct A {}
            struct B {}
        }

        goal {
            dyn Bar<A>: Bar<A>
        } yields {
            "Unique"
        }

        goal {
            dyn Bar<A>: Foo<A>
        } yields {
            "Unique"
        }

        goal {
            dyn Bar<A>: Foo<B>
        } yields {
            "No possible solution"
        }

        goal {
            exists<T> {
                dyn Bar<T>: Foo<B>
            }
        } yields {
            "Unique; substitution [?0 := B], lifetime constraints []"
        }
    }
}

#[test]
fn dyn_super_trait_cycle() {
    test! {
        program {
            trait Foo<T> where Self: Bar<T> {}
            trait Bar<T> where Self: Foo<T> {}

            struct A {}
            struct B {}
        }

        // We currently can't prove this because of the cyclic where clauses.
        // But importantly, we don't crash or get into an infinite loop.
        goal {
            dyn Bar<A>: Bar<A>
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn dyn_super_trait_not_a_cycle() {
    test! {
        program {
            trait Thing<T> {}
            trait Foo where Self: Thing<B> {}
            trait Bar where Self: Foo, Self: Thing<A> {}

            struct A {}
            struct B {}
        }

        goal {
            dyn Bar: Foo
        } yields {
            "Unique"
        }

        goal {
            dyn Bar: Thing<A>
        } yields {
            "Unique"
        }

        goal {
            dyn Bar: Thing<B>
        } yields {
            "Unique"
        }
    }
}

#[test]
fn dyn_super_trait_higher_ranked() {
    test! {
        program {
            trait Foo<'a> {}
            trait Bar<'a> where forall<'b> Self: Foo<'b> {}
            trait Baz where forall<'a> Self: Bar<'a> {}

            struct A {}
            struct B {}
        }

        goal {
            forall<'x> {
                dyn Baz: Bar<'x>
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'x> {
                dyn Baz: Foo<'x>
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'x, 'y> {
                dyn Bar<'y>: Foo<'x>
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn dyn_super_trait_non_super_trait_clause() {
    test! {
        program {
            trait Bar {}
            trait Foo where A: Bar {}

            struct A {}
            impl Bar for A {}
        }

        goal {
            dyn Foo: Foo
        } yields {
            "Unique"
        }

        goal {
            dyn Foo: Bar
        } yields {
            "No possible solution"
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
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }, \
            InEnvironment { environment: Env([]), goal: '!2_0: '!1_0 }\
            ]"
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
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!3_0: '!3_1 }, \
            InEnvironment { environment: Env([]), goal: '!3_1: '!3_0 }, \
            InEnvironment { environment: Env([]), goal: '!6_0: '!6_1 }, \
            InEnvironment { environment: Env([]), goal: '!6_1: '!6_0 }\
            ]"
        }

        // Note: these constraints are ultimately unresolveable (we
        // have to show that 'a == 'b, basically)
        goal {
            dyn forall<'c> Fn<Refs<'c, 'c>>: Eq<
                dyn forall<'a, 'b> Fn<Refs<'a, 'b>>
            >
        } yields {
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!2_0: '!2_1 }, \
            InEnvironment { environment: Env([]), goal: '!2_1: '!2_0 }, \
            InEnvironment { environment: Env([]), goal: '!5_0: '!5_1 }, \
            InEnvironment { environment: Env([]), goal: '!5_1: '!5_0 }\
            ]"
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
