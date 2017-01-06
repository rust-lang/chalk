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
                        Vec<Foo>: Clone
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
                        Foo: Clone
                    ],
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
                    value: [
                        ?0: Map<?1>
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
                        Foo: Map<Bar>
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
                        Foo: Map<Bar>
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
            "`Marker` is not implemented for `!1`"
        }

        // If we assume `T: Marker`, then obviously `T: Marker`.
        goal {
            forall<T> { if (T: Marker) { T: Marker } }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        !1: Marker
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
                        Vec<!1>: Marker
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
                    value: [
                        Vec<!1>: Clone
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
                        SomeType<!1>: Foo<u8>
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
                    value: [
                        S<S<S<S<?0>>>>: Foo
                    ],
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
                    <Vec<T> as Iterator>::Item == U
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        <Vec<!1> as Iterator>::Item == !1
                    ],
                    binders: []
                }
            }"
        }

        goal {
            forall<T> {
                <Vec<T> as Iterator>::Item == T
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        <Vec<!1> as Iterator>::Item == !1
                    ],
                    binders: []
                }
            }"
        }

        goal {
            forall<T> {
                if (<T as Iterator>::Item == u32) {
                    exists<U> {
                        <T as Iterator>::Item == U
                    }
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        <!1 as Iterator>::Item == u32
                    ],
                    binders: []
                }
            }"
        }

        goal {
            forall<T> {
                if (T: Iterator) {
                    exists<U> {
                        <T as Iterator>::Item == U
                    }
                }
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        <!1 as Iterator>::Item == (Iterator::Item)<!1>
                    ],
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
                <T as Identity>::Item == u32
            }
        } yields {
            "Solution {
                successful: Yes,
                refined_goal: Quantified {
                    value: [
                        <u32 as Identity>::Item == u32
                    ],
                    binders: []
                }
            }"
        }
    }
}
