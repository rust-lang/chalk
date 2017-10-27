#![cfg(test)]

use chalk_parse;
use errors::*;
use ir;
use lower::*;
use std::sync::Arc;

fn parse_and_lower_program(text: &str) -> Result<ir::Program> {
    chalk_parse::parse_program(text)?.lower_without_coherence()
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_parse::parse_goal(text)?.lower(program)
}

macro_rules! test {
    (program $program:tt $(goal $goal:tt with max $depth:tt yields { $expected:expr })*) => {
        solve_goal(stringify!($program), vec![$(($depth, stringify!($goal), $expected)),*])
    }
}

fn solve_goal(program_text: &str,
              goals: Vec<(usize, &str, &str)>)
{
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let program = &Arc::new(parse_and_lower_program(&program_text[1..program_text.len()-1]).unwrap());
    let env = &Arc::new(program.environment());
    ir::set_current_program(&program, || {
        for (max_size, goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = parse_and_lower_goal(&program, &goal_text[1..goal_text.len()-1]).unwrap();
            let peeled_goal = goal.into_peeled_goal();
            let result = match super::solve_root_goal(max_size, env, &peeled_goal) {
                Ok(answers) => format!("{} answer(s) found: {:#?}", answers.answers.len(), answers),
                Err(e) => format!("{:?}", e),
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
fn slg_from_env() {
    test! {
        program {
            trait Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct u32 { }
            impl Sized for u32 { }

            struct Rc<T> { }
            impl<T> Sized for Rc<T> { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }
        }

        goal {
            forall<T> { if (T: Sized) { T: Sized } }
        } with max 10 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
fn positive_cycle() {
    test! {
        program {
            trait Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Sized }
        } with max 3 yields {
            r"4 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: i32
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vec<i32>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vec<Vec<?0>>
                                    }
                                },
                                constraints: []
                            },
                            binders: [
                                U0
                            ]
                        },
                        ambiguous: true
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vec<Vec<i32>>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

/// Make sure we don't get a stack overflow or other badness for this
/// test from scalexm.
#[test]
fn subgoal_abstraction() {
    test! {
        program {
            trait Foo { }
            struct Box<T> { }
            impl<T> Foo for T where Box<T>: Foo { }
        }

        goal {
            exists<T> { T: Foo }
        } with max 50 yields {
            r"0 answer(s) found"
        }
    }
}

#[test]
fn subgoal_cycle_uninhabited() {
    test! {
        program {
            trait Foo { }
            struct Box<T> { }
            struct Vec<T> { }
            struct u32 { }
            impl<T> Foo for Box<T> where Box<Vec<T>>: Foo { }
        }

        // There is no solution here with a finite proof, so we get
        // back: 0 answer(s) found.
        goal {
            exists<T> { T: Foo }
        } with max 2 yields {
            r"0 answer(s) found: Answers {
                answers: []
            }"
        }

        // Unsurprisingly, applying negation succeeds then.
        goal {
            not { exists<T> { T: Foo } }
        } with max 2 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }

        // Eqiuvalent to the previous.
        goal {
            forall<T> { not { T: Foo } }
        } with max 2 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }

        // However, if we come across a negative goal that exceeds our
        // size threshold, we have a problem.
        goal {
            exists<T> { T = Vec<u32>, not { Vec<Vec<T>>: Foo } }
        } with max 3 yields {
            "NegativeOverflow"
        }

        // Same query with larger threshold works fine, though.
        goal {
            exists<T> { T = Vec<u32>, not { Vec<Vec<T>>: Foo } }
        } with max 4 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vec<u32>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }

        // Here, due to the hypothesis, there does indeed exist a suitable T, `U`.
        goal {
            forall<U> { if (U: Foo) { exists<T> { T: Foo } } }
        } with max 2 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: !1
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
fn subgoal_cycle_inhabited() {
    test! {
        program {
            trait Foo { }
            struct Box<T> { }
            struct Vec<T> { }
            struct u32 { }
            impl<T> Foo for Box<T> where Box<Vec<T>>: Foo { }
            impl Foo for u32 { }
        }

        goal {
            exists<T> { T: Foo }
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: u32
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
fn basic_region_constraint_from_positive_impl() {
    test! {
        program {
            trait Foo { }
            struct Ref<'a, 'b, T> { }
            struct u32 { }
            impl<'x, T> Foo for Ref<'x, 'x, T> { }
        }

        goal {
            forall<'a, 'b, T> { Ref<'a, 'b, T>: Foo }
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: [
                                    (Env(U3, []) |- LifetimeEq('!2, '!1))
                                ]
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
fn basic_region_constraint_from_unification_goal() {
    test! {
        program {
            struct Ref<'a, 'b, T> { }
            struct u32 { }
        }

        goal {
            forall<'a, 'b, T> { Ref<'a, 'b, T> = Ref<'a, 'a, T> }
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: [
                                    (Env(U3, []) |- LifetimeEq('!2, '!1))
                                ]
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn example_2_1_EWFS() {
    test! {
        program {
            trait Edge<B> { }
            trait TransitiveClosure<B> { }
            struct a { }
            struct b { }
            struct c { }

            forall<> { a: Edge<b> }
            forall<> { b: Edge<c> }
            forall<> { b: Edge<a> }
            forall<X, Y> { X: TransitiveClosure<Y> if X: Edge<Y> }
            forall<X, Y, Z> { X: TransitiveClosure<Y> if X: Edge<Z>, Z: TransitiveClosure<Y> }
        }

        goal {
            exists<V> { a: TransitiveClosure<V> }
        } with max 3 yields {
            r"3 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: a
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: b
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: c
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn example_2_2_EWFS() {
    test! {
        program {
            trait M { }
            trait P { }
            trait Q { }

            struct a { }
            struct b { }
            struct c { }

            forall<X> { X: M if not { X: P } }
            forall<> { a: P }
            forall<X> { X: P if X: Q }
            forall<> { b: Q }
            forall<X> { X: Q if X: P }
        }

        goal {
            c: M
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn example_2_3_EWFS() {
    test! {
        program {
            trait W { }
            trait M<A> { }
            trait P { }

            struct a { }
            struct b { }
            struct c { }

            forall<X, Y> { X: W if X: M<Y>, not { Y: W }, Y: P }
            forall<> { a: M<b> }
            forall<> { b: M<c> }
            forall<> { c: M<b> }
            forall<> { b: P }
        }

        goal {
            a: W
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

#[test]
#[allow(non_snake_case)]
fn example_3_3_EWFS() {
    test! {
        program {
            trait S { }
            trait P { }
            trait Q { }

            struct a { }

            forall<> { a: S if not { a: P }, not { a: Q } }
            forall<> { a: P if not { a: S }, a: Q }
            forall<> { a: Q if not { a: S }, a: P }
        }

        goal {
            a: S
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

/// Here, P is neither true nor false. If it were true, then it would
/// be false, and so forth.
#[test]
fn contradiction() {
    test! {
        program {
            trait P { }
            struct u32 { }

            forall<> { u32: P if not { u32: P } }
        }

        goal {
            u32: P
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: true
                    }
                ]
            }"
        }
    }
}

/// Here, P and Q depend on one another through a negative loop.
#[test]
fn negative_loop() {
    test! {
        program {
            trait P { }
            trait Q { }
            struct u32 { }

            forall<> { u32: P if not { u32: Q } }
            forall<> { u32: Q if not { u32: P } }
        }

        goal {
            u32: P
        } with max 3 yields {
            r"1 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {}
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: true
                    }
                ]
            }"
        }
    }
}

/// Test (along with the other `cached_answers` tests) that the
/// ordering in which we we encounter clauses doesn't affect the final
/// set of answers we get. In particular, all of them should get 5
/// answers, but in Ye Olde Days Of Yore there were sometimes bugs
/// that came up when replaying tabled answers that led to fewer
/// answers being produced.
#[test]
fn cached_answers_1() {
    test! {
        program {
            trait Sour { }
            struct Lemon { }
            struct Vinegar { }
            struct HotSauce<T> { }

            // Use explicit program clauses here rather than traits
            // and impls to avoid hashmaps and other things that
            // sometimes alter the final order of the program clauses:
            forall<> { Lemon: Sour }
            forall<> { Vinegar: Sour }
            forall<T> { HotSauce<T>: Sour if T: Sour }
        }

        goal {
            exists<T> { T: Sour }
        } with max 2 yields {
            r"5 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Lemon
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vinegar
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<?0>
                                    }
                                },
                                constraints: []
                            },
                            binders: [
                                U0
                            ]
                        },
                        ambiguous: true
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<Lemon>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<Vinegar>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

/// See `cached_answers_1`.
#[test]
fn cached_answers_2() {
    test! {
        program {
            trait Sour { }
            struct Lemon { }
            struct Vinegar { }
            struct HotSauce<T> { }

            forall<T> { HotSauce<T>: Sour if T: Sour }
            forall<> { Lemon: Sour }
            forall<> { Vinegar: Sour }
        }

        goal {
            exists<T> { T: Sour }
        } with max 2 yields {
            r"5 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Lemon
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vinegar
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<?0>
                                    }
                                },
                                constraints: []
                            },
                            binders: [
                                U0
                            ]
                        },
                        ambiguous: true
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<Lemon>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<Vinegar>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}

/// See `cached_answers_1`.
#[test]
fn cached_answers_3() {
    test! {
        program {
            trait Sour { }
            struct Lemon { }
            struct Vinegar { }
            struct HotSauce<T> { }

            forall<> { Lemon: Sour }
            forall<T> { HotSauce<T>: Sour if T: Sour }
            forall<> { Vinegar: Sour }
        }

        goal {
            exists<T> { T: Sour }
        } with max 2 yields {
            r"5 answer(s) found: Answers {
                answers: [
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Lemon
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: Vinegar
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<?0>
                                    }
                                },
                                constraints: []
                            },
                            binders: [
                                U0
                            ]
                        },
                        ambiguous: true
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<Lemon>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    },
                    Answer {
                        subst: Canonical {
                            value: ConstrainedSubst {
                                subst: Substitution {
                                    parameters: {
                                        ?0: HotSauce<Vinegar>
                                    }
                                },
                                constraints: []
                            },
                            binders: []
                        },
                        ambiguous: false
                    }
                ]
            }"
        }
    }
}
