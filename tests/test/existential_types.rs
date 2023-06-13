//! Tests related to the implied bounds rules.

use super::*;

#[test]
fn dyn_Clone_is_Clone() {
    test! {
        program {
            trait Clone { }
        }

        goal {
            forall<'s> {
                dyn Clone + 's: Clone
            }
        } yields {
            expect![["Unique"]]
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
            forall<'s> {
                dyn Clone + 's: Send
            }
        } yields {
            expect![["No possible solution"]]
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
            forall<'s> {
                (dyn Clone + Send + 's): Send
            }
        } yields {
            expect![["Unique"]]
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
            forall<'s> {
                dyn Foo<Bar> + 's: Foo<Baz>
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<'s> {
                exists<T> {
                    dyn Foo<T> + 's: Foo<Bar>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := Bar]"]]
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
            forall<'s> {
                dyn Bar<A> + 's: Bar<A>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn Bar<A> + 's: Foo<A>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn Bar<A> + 's: Foo<B>
            }
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<'s> {
                exists<T> {
                    dyn Bar<T> + 's: Foo<B>
                }
            }
        } yields {
            expect![["Unique; substitution [?0 := B]"]]
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
            forall<'s> {
                dyn Bar<A> + 's: Bar<A>
            }
        } yields {
            expect![["No possible solution"]]
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
            forall<'s> {
                dyn Bar + 's: Foo
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn Bar + 's: Thing<A>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn Bar + 's: Thing<B>
            }
        } yields {
            expect![["Unique"]]
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
            forall<'s> {
                forall<'x> {
                    dyn Baz + 's: Bar<'x>
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                forall<'x> {
                    dyn Baz + 's: Foo<'x>
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'x, 'y, 's> {
                dyn Bar<'y> + 's: Foo<'x>
            }
        } yields {
            expect![["Unique"]]
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
            forall<'s> {
                dyn Foo + 's: Foo
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn Foo + 's: Bar
            }
        } yields {
            expect![["No possible solution"]]
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
            forall<'s> {
                dyn forall<'a> Foo<Ref<'a>> + 's: Foo<Ref<'s>>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn forall<'a> Foo<Ref<'a>> + Bar + 's: Foo<Ref<'s>>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                dyn forall<'a> Foo<Ref<'a>> + Bar + 's: Bar
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<'s> {
                forall<'a> {
                    dyn Foo<Ref<'s>> + 's: Foo<Ref<'a>>
                }
            }
        } yields {
            // Note that this requires 'a == 's, so it would be resolveable later on.
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }, InEnvironment { environment: Env([]), goal: '!2_0: '!1_0 }]"]]
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
            forall<'s> {
                dyn forall<'a, 'b> Fn<Refs<'a, 'b>> + 's: Eq<
                    dyn forall<'c> Fn<Refs<'c, 'c>> + 's
                >
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!5_0: '!5_1 }, InEnvironment { environment: Env([]), goal: '!5_1: '!5_0 }]"]]
        }

        // Note: these constraints are ultimately unresolveable (we
        // have to show that 'a == 'b, basically)
        goal {
            forall<'s> {
                dyn forall<'c> Fn<Refs<'c, 'c>> + 's: Eq<
                    dyn forall<'a, 'b> Fn<Refs<'a, 'b>> + 's
                >
            }
        } yields {
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!3_0: '!3_1 }, InEnvironment { environment: Env([]), goal: '!3_1: '!3_0 }]"]]
        }

        // Note: ordering of parameters is reversed here, but that's no problem
        goal {
            forall<'s> {
                dyn forall<'c, 'd> Fn<Refs<'d, 'c>> + 's: Eq<
                    dyn forall<'a, 'b> Fn<Refs<'a, 'b>> + 's
                >
            }
        } yields {
            expect![["Unique"]]
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
            expect![["Unique; lifetime constraints [InEnvironment { environment: Env([]), goal: '!1_0: '!2_0 }, InEnvironment { environment: Env([]), goal: '!2_0: '!1_0 }]"]]
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
                forall<'s> {
                    <dyn FnOnce<(), Output = i32> + 's as FnOnce<()>>::Output = T
                }
            }
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Int(I32)]"]]
        } yields[SolverChoice::slg_default()] {
            // #234
            expect![["Ambiguous; no inference guidance"]]
        }
    }
}

#[test]
fn dyn_assoc_in_super_trait_bounds() {
    test! {
        program {
            trait Base { type Output; }
            trait Trait where Self: Base<Output = usize> {}
        }

        goal {
            forall<'s> {
                dyn Trait + 's: Trait
            }
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            forall<'s> {
                dyn Trait + 's: Base
            }
        } yields {
            expect![[r#"Unique"#]]
        }
    }
}
#[test]
fn dyn_well_formed() {
    test! {
        program {
            trait MyTrait {}
        }

        goal {
            exists<'s> {
                WellFormed(dyn MyTrait + 's)
            }
        } yields {
            expect![["Unique; for<?U0> { substitution [?0 := '^0.0] }"]]
        }
    }
}
