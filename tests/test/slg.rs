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
            let mut slg_solver = SolverChoice::SLG {
                max_size,
                max_answers: None,
            }
            .into_solver()
            .into_test();
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
