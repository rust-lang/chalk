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
                    value: [
                        Implemented(
                            Vec<Foo> as Clone
                        )
                    ],
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
                    value: [
                        Implemented(
                            Foo as Clone
                        )
                    ],
                    binders: []
                }
            }"
        }

        goal {
            Bar: Clone
        } yields {
            "`Bar as Clone` is not implemented"
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            "`Vec<Bar> as Clone` is not implemented"
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
                    value: [
                        Implemented(
                            ?0 as Map<?1>
                        )
                    ],
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
                    value: [
                        Implemented(
                            Foo as Map<Bar>
                        )
                    ],
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
                    value: [
                        Implemented(
                            Foo as Map<Bar>
                        )
                    ],
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
            "`!1 as Marker` is not implemented"
        }

        // If we assume `T: Marker`, then obviously `T: Marker`.
        goal {
            forall<T> { if (T: Marker) { T: Marker } }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        Implemented(
                            !1 as Marker
                        )
                    ],
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
                    value: [
                        Implemented(
                            Vec<!1> as Marker
                        )
                    ],
                    binders: []
                }
            }"
        }

        // Here, we don't know that `T: Clone`, so we can't prove that
        // `Vec<T>: Clone`.
        goal {
            forall<T> { Vec<T>: Clone }
        } yields {
            "`Vec<!1> as Clone` is not implemented"
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
                    value: [
                        Implemented(
                            Vec<!1> as Clone
                        )
                    ],
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
                    value: [
                        Implemented(
                            SomeType<!1> as Foo<u8>
                        )
                    ],
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
            "`!1 as Foo<?0>` is not implemented"
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
                    value: [
                        Implemented(
                            S<S<S<S<?0>>>> as Foo
                        )
                    ],
                    binders: [
                        U0
                    ]
                }
            }"
        }
    }
}
