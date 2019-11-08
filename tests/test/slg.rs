use crate::test_util::*;
use chalk_integration::db::ChalkDatabase;
use chalk_solve::ext::*;
use chalk_solve::SolverChoice;

macro_rules! test {
    (program $program:tt $(goal $goal:tt first $n:tt with max $depth:tt { $expected:expr })*) => {
        solve_goal(stringify!($program), vec![$(($depth, $n, stringify!($goal), $expected)),*])
    };

    (program $program:tt $(goal $goal:tt fixed $n:tt with max $depth:tt { $expected:expr })*) => {
        solve_goal_fixed_num_answers(
            stringify!($program),
            vec![$(($depth, $n, stringify!($goal), $expected)),*],
        )
    }
}

fn solve_goal(program_text: &str, goals: Vec<(usize, usize, &str, &str)>) {
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let db = ChalkDatabase::with(
        &program_text[1..program_text.len() - 1],
        SolverChoice::default(),
    );
    db.with_program(|_| {
        for (max_size, num_answers, goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = db
                .parse_and_lower_goal(&goal_text[1..goal_text.len() - 1])
                .unwrap();
            let peeled_goal = goal.into_peeled_goal();
            let mut slg_solver = SolverChoice::SLG { max_size }.into_solver().into_test();
            let result = format!(
                "{:#?}",
                slg_solver.force_answers(&db, &peeled_goal, num_answers)
            );
            // Strip trailing commas to handle both nightly and stable debug formatting
            let result = result.replace(",\n", "\n");
            let expected = expected.replace(",\n", "\n");
            assert_test_result_eq(&expected, &result);
        }
    });
}

fn solve_goal_fixed_num_answers(program_text: &str, goals: Vec<(usize, usize, &str, &str)>) {
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let db = ChalkDatabase::with(
        &program_text[1..program_text.len() - 1],
        SolverChoice::default(),
    );
    db.with_program(|_| {
        for (max_size, num_answers, goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = db
                .parse_and_lower_goal(&goal_text[1..goal_text.len() - 1])
                .unwrap();
            let peeled_goal = goal.into_peeled_goal();
            let mut solver = SolverChoice::SLG { max_size }.into_solver().into_test();
            let result = format!("{:?}", solver.solve(&db, &peeled_goal));

            // Strip trailing commas to handle both nightly and stable debug formatting
            let result = result.replace(",\n", "\n");
            let expected = expected.replace(",\n", "\n");
            assert_test_result_eq(&expected, &result);

            let num_cached_answers_for_goal = solver.num_cached_answers_for_goal(&db, &peeled_goal);
            // ::test_util::assert_test_result_eq(
            //     &format!("{}", num_cached_answers_for_goal),
            //     &format!("{}", expected_num_answers)
            // );
            assert_eq!(num_cached_answers_for_goal, num_answers);
        }
    });
}

#[test]
fn basic() {
    test! {
        program {
            trait Sized { }

            struct i32 { }
            impl Sized for i32 { }
        }

        goal {
            forall<T> { if (T: Sized) { T: Sized } }
        } first 2 with max 10 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }
    }
}

#[test]
fn breadth_first() {
    test! {
        program {
            trait Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Sized }
        } first 5 with max 10 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := i32],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vec<i32>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Slice<i32>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vec<Vec<i32>>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Slice<Vec<i32>>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }
    }
}

#[test]
fn infinite_recursion() {
    test! {
        program {
            trait A { }
            trait B { }
            trait C { }
            trait D { }

            struct Vec<T> { }
            impl<T> A for Vec<T> where T: B { }
            impl<T> B for Vec<T> where T: C { }
            impl<T> C for Vec<T> where T: D { }
            impl<T> D for Vec<T> where T: A { }
        }

        goal {
            exists<T> { T: A }
        } first 5 with max 10 {
            r"[]"
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
        } first 5 with max 50 {
            r"[]"
        }
    }
}

#[test]
fn flounder() {
    test! {
        program {
            trait A { }

            struct Vec<T> { }
            impl<T> A for Vec<T> { }
        }

        goal {
            exists<T> { not { T: A } }
        } first 5 with max 10 {
            "Floundered"
        }
    }
}

// Test that, when solving `?T: Sized`, we only wind up pulling a few
// answers before we stop.
#[test]
fn only_draw_so_many() {
    test! {
        program {
            trait Sized { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Sized }
        } fixed 2 with max 10 {
            "Some(Ambig(Unknown))"
        }
    }
}

#[test]
fn only_draw_so_many_blow_up() {
    test! {
        program {
            trait Sized { }
            trait Foo { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }
            impl<T> Foo for Vec<T> where T: Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
        }

        goal {
            exists<T> { T: Foo }
        } fixed 767 with max 10 {
            // FIXME: should be 2?
            "Some(Ambig(Definite(Canonical { value: [?0 := Vec<^0>], binders: [Ty(U0)] })))"
        }
    }
}

/// Here, P and Q depend on one another through a negative loop.
#[test]
#[should_panic(expected = "negative cycle")]
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
        } first 5 with max 3 {
            // Negative cycle -> panic
            r""
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
        } first 10 with max 2 {
            r"[]"
        }

        // Unsurprisingly, applying negation succeeds then.
        goal {
            not { exists<T> { T: Foo } }
        } first 10 with max 2 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }

        // Equivalent to the previous.
        goal {
            forall<T> { not { T: Foo } }
        } first 10 with max 2 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }

        // However, if we come across a negative goal that exceeds our
        // size threshold, we have a problem.
        goal {
            exists<T> { T = Vec<u32>, not { Vec<Vec<T>>: Foo } }
        } first 10 with max 3 {
            "Floundered"
        }

        // Same query with larger threshold works fine, though.
        goal {
            exists<T> { T = Vec<u32>, not { Vec<Vec<T>>: Foo } }
        } first 10 with max 4 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vec<u32>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }

        // Here, due to the hypothesis, there does indeed exist a suitable T, `U`.
        goal {
            forall<U> { if (U: Foo) { exists<T> { T: Foo } } }
        } first 10 with max 2 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := !1_0],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
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
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := u32],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
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
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [],
                            constraints: [
                                InEnvironment {
                                    environment: Env([]),
                                    goal: '!1_1 == '!1_0
                                }
                            ]
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
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
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := b],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := c],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := a],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
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
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }
    }
}

#[test]
#[should_panic(expected = "negative cycle")]
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
        } first 10 with max 3 {
            // Negative cycle -> panic
            r""
        }
    }
}

#[test]
#[should_panic(expected = "negative cycle")]
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
        } first 10 with max 3 {
            // Negative cycle -> panic
            r""
        }
    }
}

/// Here, P is neither true nor false. If it were true, then it would
/// be false, and so forth.
#[test]
#[should_panic(expected = "negative cycle")]
fn contradiction() {
    test! {
        program {
            trait P { }
            struct u32 { }

            forall<> { u32: P if not { u32: P } }
        }

        goal {
            u32: P
        } first 10 with max 3 {
            // Negative cycle -> panic
            r""
        }
    }
}

/// Test (along with the other `cached_answers` tests) that the
/// ordering in which we we encounter clauses doesn't affect the final
/// set of answers we get. In particular, all of them should get 5
/// answers, but in Ye Olde Days Of Yore there were sometimes bugs
/// that came up when replaying tabled answers that led to fewer
/// answers being produced.
///
/// This test is also a test for ANSWER ABSTRACTION: the only reason
/// we get 5 answers is because of the max size of 2.
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
        } first 10 with max 2 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Lemon],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vinegar],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Lemon>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Vinegar>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<^0>],
                            constraints: []
                        },
                        binders: [
                            Ty(U0)
                        ]
                    },
                    ambiguous: true
                }
            ]"
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
        } first 10 with max 2 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Lemon],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vinegar],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Lemon>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Vinegar>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<^0>],
                            constraints: []
                        },
                        binders: [
                            Ty(U0)
                        ]
                    },
                    ambiguous: true
                }
            ]"
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
        } first 10 with max 2 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Lemon],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Lemon>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vinegar],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<^0>],
                            constraints: []
                        },
                        binders: [
                            Ty(U0)
                        ]
                    },
                    ambiguous: true
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Vinegar>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }
    }
}

/// Here, P depends on Q negatively, but Q depends only on itself.
#[test]
#[should_panic(expected = "negative cycle")]
fn negative_answer_ambiguous() {
    test! {
        program {
            trait P { }
            trait Q { }
            struct u32 { }

            forall<> { u32: P if not { u32: Q } }
            forall<> { u32: Q if not { u32: Q } }
        }

        goal {
            u32: P
        } first 10 with max 3 {
            // Negative cycle -> panic
            r""
        }
    }
}

#[test]
fn non_enumerable_traits_direct() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable { }
            impl NonEnumerable for Foo { }
            impl NonEnumerable for Bar { }

            trait Enumerable { }
            impl Enumerable for Foo { }
            impl Enumerable for Bar { }
        }

        goal {
            exists<A> { A: NonEnumerable }
        } first 10 with max 3 {
            r"Floundered"
        }

        goal {
            exists<A> { A: Enumerable }
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Foo]
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Bar]
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
            ]"
        }

        goal {
            Foo: NonEnumerable
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: []
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
            ]"
        }
    }
}

#[test]
fn non_enumerable_traits_indirect() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable { }
            impl NonEnumerable for Foo { }
            impl NonEnumerable for Bar { }

            trait Debug { }
            impl<T> Debug for T where T: NonEnumerable { }
        }

        goal {
            exists<A> { A: Debug }
        } first 10 with max 3 {
            r"Floundered"
        }
    }
}

#[test]
fn non_enumerable_traits_double() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable1 { }
            impl NonEnumerable1 for Foo { }
            impl NonEnumerable1 for Bar { }

            #[non_enumerable]
            trait NonEnumerable2 { }
            impl NonEnumerable2 for Foo { }
            impl NonEnumerable2 for Bar { }

            trait Debug { }
            impl<T> Debug for T where T: NonEnumerable1, T: NonEnumerable2  { }
        }

        goal {
            exists<A> { A: Debug }
        } first 10 with max 3 {
            r"Floundered"
        }
    }
}

#[test]
fn non_enumerable_traits_reorder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[non_enumerable]
            trait NonEnumerable { }
            impl NonEnumerable for Foo { }
            impl NonEnumerable for Bar { }

            trait Enumerable { }
            impl Enumerable for Foo { }

            // In this test, we first try to solve to solve `T:
            // NonEnumerable` but then we discover it's
            // non-enumerable, and so we push it off for later. Then
            // we try to solve the `T: Enumerable` trait.

            trait Debug1 { }
            impl<T> Debug1 for T where T: Enumerable, T: NonEnumerable { }

            trait Debug2 { }
            impl<T> Debug2 for T where T: NonEnumerable, T: Enumerable { }
        }

        goal {
            exists<A> { A: Debug1 }
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Foo]
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
            ]"
        }


        goal {
            exists<A> { A: Debug2 }
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Foo]
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
            ]"
        }
    }
}

#[test]
fn auto_traits_flounder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[auto]
            trait Send { }
        }

        goal {
            exists<A> { A: Send }
        } first 10 with max 3 {
            r"Floundered"
        }
    }
}

#[test]
fn negative_reorder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            trait IsFoo { }
            impl IsFoo for Foo { }

            trait Enumerable { }
            impl Enumerable for Foo { }
            impl Enumerable for Bar { }

            // In this test, we first try to solve to solve `not { T:
            //  IsFoo }` but then we discover it's
            // non-enumerable, and so we push it off for later. Then
            // we try to solve the `T: Enumerable` trait.

            trait Debug1 { }
            forall<T> {
                T: Debug1 if T: Enumerable, not { T: IsFoo }
            }

            trait Debug2 { }
            forall<T> {
                T: Debug2 if not { T: IsFoo }, T: Enumerable
            }
        }

        goal {
            exists<A> { A: Debug1 }
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Bar]
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
            ]"
        }


        goal {
            exists<A> { A: Debug2 }
        } first 10 with max 3 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Bar]
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
            ]"
        }
    }
}

/// Test a tricky case for coinductive handling:
///
/// While proving C1, we try to prove C2, which recursively requires
/// proving C1.  If you are naive, you will assume that C2 therefore
/// holds -- but this is wrong, because C1 later fails when proving
/// C3.
#[test]
fn coinductive_unsound1() {
    test! {
        program {
            trait C1orC2 { }

            #[auto]
            trait C1 { }

            #[auto]
            trait C2 { }

            #[auto]
            trait C3 { }

            forall<T> {
                T: C1 if T: C2, T: C3
            }

            forall<T> {
                T: C2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C2
            }
        }

        goal {
            forall<X> { X: C1orC2 }
        } first 10 with max 3 {
            // FIXME(chalk#248) -- demonstrate bug in coinduction
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [],
                            constraints: []
                        }
                        binders: []
                    }
                    ambiguous: false
                }
           ]"
        }
    }
}

/// The only difference between this test and `coinductive_unsound1`
/// is the order of the final `forall` clauses.
#[test]
fn coinductive_unsound2() {
    test! {
        program {
            trait C1orC2 { }

            #[auto]
            trait C1 { }

            #[auto]
            trait C2 { }

            #[auto]
            trait C3 { }

            forall<T> {
                T: C1 if T: C2, T: C3
            }

            forall<T> {
                T: C2 if T: C1
            }

            forall<T> {
                T: C1orC2 if T: C2
            }

            forall<T> {
                T: C1orC2 if T: C1
            }
        }

        goal {
            forall<X> { X: C1orC2 }
        } first 10 with max 3 {
            r"[]"
        }
    }
}
