//! Tests related to the implied bounds rules.

use super::*;

#[test]
fn dyn_Clone_is_Clone() {
    test! {
        program {
            trait Clone { }
        }

        goal {
            forall<'static> {
                dyn Clone + 'static: Clone
            }
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
            forall<'static> {
                dyn Clone + 'static: Send
            }
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
            forall<'static> {
                (dyn Clone + Send + 'static): Send
            }
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
            forall<'static> {
                dyn Foo<Bar> + 'static: Foo<Baz>
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<'static> {
                exists<T> {
                    dyn Foo<T> + 'static: Foo<Bar>
                }
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
            forall<'static> {
                dyn Bar<A> + 'static: Bar<A>
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'static> {
                dyn Bar<A> + 'static: Foo<A>
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'static> {
                dyn Bar<A> + 'static: Foo<B>
            }
        } yields {
            "No possible solution"
        }

        goal {
            forall<'static> {
                exists<T> {
                    dyn Bar<T> + 'static: Foo<B>
                }
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
            forall<'static> {
                dyn Bar<A> + 'static: Bar<A>
            }
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
            forall<'static> {
                dyn Bar + 'static: Foo
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'static> {
                dyn Bar + 'static: Thing<A>
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'static> {
                dyn Bar + 'static: Thing<B>
            }
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
            forall<'static> {
                forall<'x> {
                    dyn Baz + 'static: Bar<'x>
                }
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'static> {
                forall<'x> {
                    dyn Baz + 'static: Foo<'x>
                }
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'x, 'y, 'static> {
                dyn Bar<'y> + 'static: Foo<'x>
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
            forall<'static> {
                dyn Foo + 'static: Foo
            }
        } yields {
            "Unique"
        }

        goal {
            forall<'static> {
                dyn Foo + 'static: Bar
            }
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
                dyn forall<'a> Foo<Ref<'a>> + 'static: Foo<Ref<'static>>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'static> {
                dyn forall<'a> Foo<Ref<'a>> + Bar + 'static: Foo<Ref<'static>>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'static> {
                dyn forall<'a> Foo<Ref<'a>> + Bar + 'static: Bar
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            forall<'static> {
                forall<'a> {
                    dyn Foo<Ref<'static>> + 'static: Foo<Ref<'a>>
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
            forall<'static> {
                dyn forall<'a, 'b> Fn<Refs<'a, 'b>> + 'static: Eq<
                    dyn forall<'c> Fn<Refs<'c, 'c>> + 'static
                >
            }
        } yields {
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!4_0: '!4_1 }, \
            InEnvironment { environment: Env([]), goal: '!4_1: '!4_0 }, \
            InEnvironment { environment: Env([]), goal: '!7_0: '!7_1 }, \
            InEnvironment { environment: Env([]), goal: '!7_1: '!7_0 }\
            ]"
        }

        // Note: these constraints are ultimately unresolveable (we
        // have to show that 'a == 'b, basically)
        goal {
            forall<'static> {
                dyn forall<'c> Fn<Refs<'c, 'c>> + 'static: Eq<
                    dyn forall<'a, 'b> Fn<Refs<'a, 'b>> + 'static
                >
            }
        } yields {
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!3_0: '!3_1 }, \
            InEnvironment { environment: Env([]), goal: '!3_1: '!3_0 }, \
            InEnvironment { environment: Env([]), goal: '!6_0: '!6_1 }, \
            InEnvironment { environment: Env([]), goal: '!6_1: '!6_0 }\
            ]"
        }

        // Note: ordering of parameters is reversed here, but that's no problem
        goal {
            forall<'static> {
                dyn forall<'c, 'd> Fn<Refs<'d, 'c>> + 'static: Eq<
                    dyn forall<'a, 'b> Fn<Refs<'a, 'b>> + 'static
                >
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn dyn_lifetime_bound() {
    test! {
        program {
            trait Foo { }

            trait Eq<A> { }

            impl<A> Eq<A> for A { }
        }

        goal {
            forall<'a> {
                forall<'b> {
                    dyn Foo + 'a: Eq<dyn Foo + 'b>
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints [\
            InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }, \
            InEnvironment { environment: Env([]), goal: '!2_0: '!1_0 }\
            ]"
        }
    }
}

#[test]
fn dyn_associated_type_binding() {
    test! {
        program {
            trait FnOnce<Args> { type Output; }
        }

        goal {
            exists<T> {
                forall<'static> {
                    <dyn FnOnce<(), Output = i32> + 'static as FnOnce<()>>::Output = T
                }
            }
        } yields[SolverChoice::recursive()] {
            "Unique; substitution [?0 := Int(I32)], lifetime constraints []"
        } yields[SolverChoice::slg_default()] {
            // #234
            "Ambiguous"
        }
    }
}
