use chalk_rust_parse;
use errors::*;
use ir;
use lower::*;
use solve::goal::Prove;
use solve::solver::Solver;
use std::sync::Arc;

fn parse_and_lower_program(text: &str) -> Result<ir::Program> {
    chalk_rust_parse::parse_program(text)?.lower()
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_rust_parse::parse_goal(text)?.lower(program)
}

macro_rules! test {
    (program $program:tt $(goal $goal:tt yields { $expected:expr })*) => {
        solve_goal(stringify!($program), vec![$((stringify!($goal), $expected)),*])
    }
}

fn solve_goal(program_text: &str,
              goals: Vec<(&str, &str)>)
{
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let program = Arc::new(parse_and_lower_program(&program_text[1..program_text.len()-1]).unwrap());
    ir::set_current_program(&program, || {
        for (goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = parse_and_lower_goal(&program, &goal_text[1..goal_text.len()-1]).unwrap();

            // Pick a low overflow depth just because the existing
            // tests don't require a higher one.
            let overflow_depth = 3;

            let mut solver = Solver::new(&program, overflow_depth);
            let result = match Prove::new(&mut solver, goal).solve() {
                Ok(v) => format!("{:#?}", v),
                Err(e) => format!("{}", e),
            };
            println!("expected:\n{}", expected);
            println!("actual:\n{}", result);

            // remove all whitespace:
            let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
            let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
            assert!(!expected1.is_empty() && result1.starts_with(&expected1));
        }
    });
}

#[test]
fn prove_clone() {
    test! {
        program {
            struct Foo { }
            struct Bar { }
            struct Vec<T> { }
            trait Clone { }
            impl<T> Clone for Vec<T> where T: Clone { }
            impl Clone for Foo { }
        }

        goal {
            Vec<Foo>: Clone
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Vec<Foo>: Clone
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        goal {
            Foo: Clone
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Foo: Clone
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        goal {
            Bar: Clone
        } yields {
            "`Clone` is not implemented for `Bar`"
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            "`Clone` is not implemented for `Vec<Bar>`"
        }
    }
}

#[test]
fn prove_infer() {
    test! {
        program {
            struct Foo { }
            struct Bar { }
            trait Map<T> { }
            impl Map<Bar> for Foo { }
            impl Map<Foo> for Bar { }
        }

        goal {
            exists<A, B> { A: Map<B> }
        } yields {
            "Solution {
                successful: Maybe,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            ?0: Map<?1>
                        ],
                        constraints: []
                    },
                    binders: [
                        U0,
                        U0
                    ]
                }
            }"
        }

        goal {
            exists<A> { A: Map<Bar> }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Foo: Map<Bar>
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        goal {
            exists<A> { Foo: Map<A> }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Foo: Map<Bar>
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }
    }
}

#[test]
fn prove_forall() {
    test! {
        program {
            struct Foo { }
            struct Vec<T> { }

            trait Marker { }
            impl<T> Marker for Vec<T> { }

            trait Clone { }
            impl<T> Clone for Vec<T> where T: Clone { }
        }

        goal {
            forall<T> { T: Marker }
        } yields {
            "`Marker` is not implemented for `!1`"
        }

        // If we assume `T: Marker`, then obviously `T: Marker`.
        goal {
            forall<T> { if (T: Marker) { T: Marker } }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            !1: Marker
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        // We don't have know to anything about `T` to know that
        // `Vec<T>: Marker`.
        goal {
            forall<T> { Vec<T>: Marker }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Vec<!1>: Marker
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        // Here, we don't know that `T: Clone`, so we can't prove that
        // `Vec<T>: Clone`.
        goal {
            forall<T> { Vec<T>: Clone }
        } yields {
            "`Clone` is not implemented for `Vec<!1>`"
        }

        // Here, we do know that `T: Clone`, so we can.
        goal {
            forall<T> {
                if (T: Clone) {
                    Vec<T>: Clone
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Vec<!1>: Clone
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }
    }
}

#[test]
fn higher_ranked() {
    test! {
        program {
            struct u8 { }
            struct SomeType<T> { }
            trait Foo<T> { }
            impl<U> Foo<u8> for SomeType<U> { }
        }

        goal {
            exists<V> {
                forall<U> {
                    SomeType<U>: Foo<V>
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            SomeType<!1>: Foo<u8>
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }
    }
}

#[test]
fn ordering() {
    test! {
        program {
            trait Foo<T> { }
            impl<U> Foo<U> for U { }
        }

        goal {
            exists<V> {
                forall<U> {
                    U: Foo<V>
                }
            }
        } yields {
            "`Foo<?0>` is not implemented for `!1`"
        }
    }
}

/// This test forces the solver into an overflow scenario: `Foo` is
/// only implemented for `S<S<S<...>>>` ad infinitum. So when asked to
/// compute the type for which `Foo` is implemented, we wind up
/// recursing for a while before we overflow. You can see that our
/// final result is "Maybe" (i.e., either multiple proof trees or an
/// infinite proof tree) and that we do conclude that, if a definite
/// proof tree exists, it must begin with `S<S<S<S<...>>>>`.
#[test]
fn max_depth() {
    test! {
        program {
            trait Foo { }
            struct S<T> { }
            impl<T> Foo for S<T> where T: Foo { }
        }

        goal {
            exists<T> {
                T: Foo
            }
        } yields {
            "Solution {
                successful: Maybe,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            S<S<S<S<?0>>>>: Foo
                        ],
                        constraints: []
                    },
                    binders: [
                        U0
                    ]
                }
            }"
        }
    }
}

#[test]
fn normalize() {
    test! {
        program {
            trait Iterator { type Item; }
            struct Vec<T> { }
            struct u32 { }
            impl<T> Iterator for Vec<T> {
                type Item = T;
            }
        }

        goal {
            forall<T> {
                exists<U> {
                    Vec<T>: Iterator<Item = U>
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            <Vec<!1> as Iterator>::Item ==> !1
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        goal {
            forall<T> {
                Vec<T>: Iterator<Item = T>
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            <Vec<!1> as Iterator>::Item ==> !1
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        goal {
            forall<T> {
                if (T: Iterator<Item = u32>) {
                    exists<U> {
                        T: Iterator<Item = U>
                    }
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            <!1 as Iterator>::Item ==> u32
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    exists<U> {
                        T: Iterator<Item = U>
                    }
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            <!1 as Iterator>::Item ==> (Iterator::Item)<!1>
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
#[test]
fn normalize_rev_infer() {
    test! {
        program {
            trait Identity { type Item; }
            struct u32 { }
            struct i32 { }
            impl Identity for u32 { type Item = u32; }
            impl Identity for i32 { type Item = i32; }
        }

        goal {
            exists<T> {
                T: Identity<Item = u32>
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            <u32 as Identity>::Item ==> u32
                        ],
                        constraints: []
                    },
                    binders: []
                }
            }"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
#[test]
fn region_equality() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            forall<'a, 'b> {
                Ref<'a, Unit>: Eq<Ref<'b, Unit>>
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Ref<'!1, Unit>: Eq<Ref<'!2, Unit>>
                        ],
                        constraints: [
                            LifetimeEq(
                                '!2,
                                '!1
                            )
                        ]
                    },
                    binders: []
                }
            }"
        }

        goal {
            forall<'a> {
                exists<'b> {
                    Ref<'a, Unit>: Eq<Ref<'b, Unit>>
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            Ref<'!1, Unit>: Eq<Ref<'?0, Unit>>
                        ],
                        constraints: [
                            LifetimeEq(
                                '?1,
                                '!1
                            )
                        ]
                    },
                    binders: [
                        U1,
                        U1
                    ]
                }
            }"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
#[test]
fn forall_equality() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            // A valid equality; we get back a series of solvable
            // region constraints, since each region variable must
            // refer to exactly one skolemized region, and they are
            // all in a valid universe to do so (universe 4).
            //
            // I'm not quite sure why we get six lifetime constraints,
            // though.
            for<'a, 'b> Ref<'a, Ref<'b, Unit>>: Eq<for<'c, 'd> Ref<'c, Ref<'d, Unit>>>
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            for<2> Ref<'?0, Ref<'?1, Unit>>: Eq<for<2> Ref<'?0, Ref<'?1, Unit>>>
                        ],
                        constraints: [
                            LifetimeEq(
                                '!1,
                                '?0
                            ),
                            LifetimeEq(
                                '!1,
                                '?1
                            ),
                            LifetimeEq(
                                '!1,
                                '?2
                            ),
                            LifetimeEq(
                                '!2,
                                '?3
                            ),
                            LifetimeEq(
                                '!2,
                                '?4
                            ),
                            LifetimeEq(
                                '!2,
                                '?5
                            )
                        ]
                    },
                    binders: [
                        U2,
                        U2,
                        U2,
                        U2,
                        U2,
                        U2
                    ]
                }
            }"
        }

        goal {
            // Note: this equality is false, but we get back successful;
            // this is because the region constraints are unsolvable.
            //
            // Note that `?0` (in universe 4) must be equal to both
            // `!3` and `!4`, which of course it cannot be.
            for<'a, 'b> Ref<'a, Ref<'b, Ref<'a, Unit>>>: Eq<
                for<'c, 'd> Ref<'c, Ref<'d, Ref<'d, Unit>>>>
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            for<2> Ref<'?0, Ref<'?1, Ref<'?0, Unit>>>: Eq<for<2> Ref<'?0, Ref<'?1, Ref<'?1, Unit>>>>
                        ],
                        constraints: [
                            LifetimeEq(
                                '!1,
                                '?0
                            ),
                            LifetimeEq(
                                '!1,
                                '?1
                            ),
                            LifetimeEq(
                                '!1,
                                '?2
                            ),
                            LifetimeEq(
                                '!1,
                                '?3
                            ),
                            LifetimeEq(
                                '!2,
                                '?0
                            ),
                            LifetimeEq(
                                '!2,
                                '?4
                            ),
                            LifetimeEq(
                                '!2,
                                '?5
                            )
                        ]
                    },
                    binders: [
                        U2,
                        U2,
                        U2,
                        U2,
                        U2,
                        U2
                    ]
                }
            }"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.  Produces
/// a pretty convoluted set of lifetime constraints; seems clear that
/// we can do some simplification and/or need to change the structure.
#[test]
fn forall_projection() {
    test! {
        program {
            trait Eq<T> { }
            impl<T> Eq<T> for T { }

            trait DropLt<'a> { type Item; }
            impl<'a, T> DropLt<'a> for T { type Item = T; }

            struct Unit { }
            struct Ref<'a, T> { }
        }

        goal {
            for<'a> <Unit as DropLt<'a>>::Item: Eq<Unit>
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: Constrained {
                        value: [
                            for<1> <Unit as DropLt<'?0>>::Item: Eq<Unit>
                        ],
                        constraints: [
                            LifetimeEq(
                                '?0,
                                '?1
                            ),
                            LifetimeEq(
                                '?2,
                                '?3
                            ),
                            LifetimeEq(
                                '?2,
                                '?4
                            ),
                            LifetimeEq(
                                '?3,
                                '?5
                            ),
                            LifetimeEq(
                                '?1,
                                '?6
                            ),
                            LifetimeEq(
                                '?4,
                                '?7
                            ),
                            LifetimeEq(
                                '!1,
                                '?8
                            ),
                            LifetimeEq(
                                '!1,
                                '?9
                            ),
                            LifetimeEq(
                                '!1,
                                '?10
                            ),
                            LifetimeEq(
                                '!1,
                                '?11
                            ),
                            LifetimeEq(
                                '!1,
                                '?12
                            ),
                            LifetimeEq(
                                '!1,
                                '?13
                            ),
                            LifetimeEq(
                                '!1,
                                '!1
                            )
                        ]
                    },
                    binders: [
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1,
                        U1
                    ]
                }
            }"
        }
    }
}
