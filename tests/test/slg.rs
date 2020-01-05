use crate::test_util::*;
use chalk_integration::db::ChalkDatabase;
use chalk_solve::ext::*;
use chalk_solve::SolverChoice;

macro_rules! test {
    (program $program:tt $(goal $goal:tt first $n:tt with max $depth:tt { $expected:expr })*) => {
        solve_goal(stringify!($program), vec![$(($depth, $n, stringify!($goal), $expected)),*])
    };
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
            let mut slg_solver = SolverChoice::SLG { max_size, max_answers: None }.into_solver().into_test();
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
                CompleteAnswer {
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
                CompleteAnswer {
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
                CompleteAnswer {
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
                CompleteAnswer {
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
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Lemon],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vinegar],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Lemon>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Vinegar>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
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
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Lemon],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vinegar],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Lemon>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Vinegar>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
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
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Lemon],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := HotSauce<Lemon>],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Vinegar],
                            constraints: []
                        },
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
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
                CompleteAnswer {
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
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Foo]
                            constraints: []
                        }
                        binders: []
                    },
                    ambiguous: false
                },
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: [?0 := Bar]
                            constraints: []
                        }
                        binders: []
                    },
                    ambiguous: false
                }
            ]"
        }

        goal {
            Foo: NonEnumerable
        } first 10 with max 3 {
            r"[
                CompleteAnswer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: []
                            constraints: []
                        }
                        binders: []
                    },
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
