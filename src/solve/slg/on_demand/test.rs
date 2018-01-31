#![cfg(test)]

use chalk_parse;
use errors::*;
use ir;
use lower::*;
use solve::slg::on_demand::forest::Forest;
use std::sync::Arc;

fn parse_and_lower_program(text: &str) -> Result<ir::Program> {
    chalk_parse::parse_program(text)?.lower_without_coherence()
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_parse::parse_goal(text)?.lower(program)
}

macro_rules! test {
    (program $program:tt $(goal $goal:tt first $n:tt with max $depth:tt { $expected:expr })*) => {
        solve_goal(stringify!($program), vec![$(($depth, $n, stringify!($goal), $expected)),*])
    }
}

fn solve_goal(program_text: &str, goals: Vec<(usize, usize, &str, &str)>) {
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));
    let program =
        &Arc::new(parse_and_lower_program(&program_text[1..program_text.len() - 1]).unwrap());
    let env = &Arc::new(program.environment());
    ir::set_current_program(&program, || {
        for (max_size, num_answers, goal_text, expected) in goals {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = parse_and_lower_goal(&program, &goal_text[1..goal_text.len() - 1]).unwrap();
            let peeled_goal = goal.into_peeled_goal();
            let mut forest = Forest::new(env, max_size);
            let result = format!("{:#?}", forest.force_answers(peeled_goal, num_answers));

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
                            subst: Substitution {
                                parameters: {}
                            },
                            constraints: []
                        },
                        binders: []
                    },
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: []
                    }
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
                            subst: Substitution {
                                parameters: {
                                    ?0: i32
                                }
                            },
                            constraints: []
                        },
                        binders: []
                    },
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: []
                    }
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
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: []
                    }
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: Substitution {
                                parameters: {
                                    ?0: Slice<i32>
                                }
                            },
                            constraints: []
                        },
                        binders: []
                    },
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: []
                    }
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
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: []
                    }
                },
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: Substitution {
                                parameters: {
                                    ?0: Slice<Vec<i32>>
                                }
                            },
                            constraints: []
                        },
                        binders: []
                    },
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: []
                    }
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

#[test]
fn flounder() {
    test! {
        program {
            trait A { }

            struct Vec<T> { }
            impl<T> A for Vec<T> { }
        }

        goal {
            // This goal "flounders" because it has a free existential
            // variable. We choose to replace it with a `CannotProve`
            // result.
            exists<T> { not { T: A } }
        } first 5 with max 10 {
            r"[
                Answer {
                    subst: Canonical {
                        value: ConstrainedSubst {
                            subst: Substitution {
                                parameters: {
                                    ?0: ?0
                                }
                            },
                            constraints: []
                        },
                        binders: [
                            Ty(U0)
                        ]
                    },
                    delayed_literals: DelayedLiteralSet {
                        delayed_literals: [
                            CannotProve(
                                ()
                            )
                        ]
                    }
                }
            ]"
        }
    }
}

// Test that, when solving `?T: Sized`, we only wind up pulling a few
// answers before we stop.
#[test]
fn only_draw_so_many() {
    let program_text = "
            trait Sized { }

            struct Vec<T> { }
            impl<T> Sized for Vec<T> where T: Sized { }

            struct i32 { }
            impl Sized for i32 { }

            struct Slice<T> { }
            impl<T> Sized for Slice<T> where T: Sized { }
    ";

    let goal_text = "exists<T> { T: Sized }";

    let program = &Arc::new(parse_and_lower_program(program_text).unwrap());
    let env = &Arc::new(program.environment());
    ir::set_current_program(&program, || {
        let goal = parse_and_lower_goal(&program, goal_text).unwrap();
        let peeled_goal = goal.into_peeled_goal();
        let mut forest = Forest::new(env, 10);
        let solution = forest.solve(&peeled_goal);

        // First, check we got the expected solution.
        assert_eq!(format!("{:?}", solution), "Some(Ambig(Unknown))");

        // Next, check how many answers we had to peel to get it.
        let table = forest.get_or_create_table_for_ucanonical_goal(peeled_goal.clone());
        assert_eq!(forest.tables[table].num_cached_answers(), 2);
    });
}
