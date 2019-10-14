#![cfg(test)]

use crate::db::ChalkDatabase;
use crate::query::LoweringDatabase;
use chalk_ir;
use chalk_solve::ext::*;
use chalk_solve::{Solution, SolverChoice};

#[cfg(feature = "bench")]
mod bench;
mod coherence;
mod slg;
mod wf_lowering;

fn assert_result(result: &Option<Solution>, expected: &str) {
    let result = match result {
        Some(v) => format!("{}", v),
        None => format!("No possible solution"),
    };

    println!("expected:\n{}", expected);
    println!("actual:\n{}", result);

    let expected1: String = expected.chars().filter(|w| !w.is_whitespace()).collect();
    let result1: String = result.chars().filter(|w| !w.is_whitespace()).collect();
    assert!(!expected1.is_empty() && result1.starts_with(&expected1));
}

macro_rules! test {
    (program $program:tt $($goals:tt)*) => {
        test!(@program[$program]
              @parsed_goals[]
              @unparsed_goals[$($goals)*])
    };

    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[]) => {
        solve_goal(stringify!($program), vec![$($parsed_goals),*])
    };

    // goal { G } yields { "Y" } -- test both solvers behave the same (the default)
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt yields { $expected:expr }
        $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[
                  $($parsed_goals)*
                      (stringify!($goal), SolverChoice::default(), $expected)
              ]
              @unparsed_goals[$($unparsed_goals)*])
    };

    // goal { G } yields[C1] { "Y1" } yields[C2] { "Y2" } -- test that solver C1 yields Y1
    // and C2 yields Y2
    //
    // Annoyingly, to avoid getting a parsing ambiguity error, we have
    // to distinguish the case where there are other goals to come
    // (this rule) for the last goal in the list (next rule). There
    // might be a more elegant fix than copy-and-paste but this works.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt $(yields[$($C:expr),+] { $expected:expr })*
            goal $($unparsed_goals:tt)*
    ]) => {
        test!(@program[$program]
              @parsed_goals[$($parsed_goals)*
                            $($((stringify!($goal), $C, $expected))+)+]
              @unparsed_goals[goal $($unparsed_goals)*])
    };

    // same as above, but for the final goal in the list.
    (@program[$program:tt] @parsed_goals[$($parsed_goals:tt)*] @unparsed_goals[
        goal $goal:tt $(yields[$($C:expr),+] { $expected:expr })*
    ]) => {
        test!(@program[$program]
              @parsed_goals[$($parsed_goals)*
                            $($((stringify!($goal), $C, $expected))+)+]
              @unparsed_goals[])
    };
}

fn solve_goal(program_text: &str, goals: Vec<(&str, SolverChoice, &str)>) {
    println!("program {}", program_text);
    assert!(program_text.starts_with("{"));
    assert!(program_text.ends_with("}"));

    let mut db = ChalkDatabase::with(
        &program_text[1..program_text.len() - 1],
        SolverChoice::default(),
    );

    for (goal_text, solver_choice, expected) in goals {
        if db.solver_choice() != solver_choice {
            db.set_solver_choice(solver_choice);
        }

        let program = db.checked_program().unwrap();

        chalk_ir::tls::set_current_program(&program, || {
            println!("----------------------------------------------------------------------");
            println!("goal {}", goal_text);
            assert!(goal_text.starts_with("{"));
            assert!(goal_text.ends_with("}"));
            let goal = db
                .parse_and_lower_goal(&goal_text[1..goal_text.len() - 1])
                .unwrap();

            println!("using solver: {:?}", solver_choice);
            let peeled_goal = goal.into_peeled_goal();
            let result = db.solve(&peeled_goal);
            assert_result(&result, expected);
        });
    }
}

mod coherence_goals;
mod coinduction;
mod cycle;
mod implied_bounds;
mod impls;
mod negation;
mod projection;
mod unify;
mod wf_goals;

#[test]
fn auto_trait_without_impls() {
    test! {
        program {
            #[auto] trait Send { }

            struct i32 { }

            struct Useless<T> { }

            struct Data<T> {
                data: T
            }
        }

        goal {
            i32: Send
        } yields {
            "Unique"
        }

        // No fields so `Useless<T>` is `Send`.
        goal {
            forall<T> {
                Useless<T>: Send
            }
        } yields {
            "Unique"
        }

        goal {
            forall<T> {
                if (T: Send) {
                    Data<T>: Send
                }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn auto_trait_with_impls() {
    test! {
        program {
            #[auto] trait Send { }

            struct i32 { }
            struct f32 { }
            struct Vec<T> { }

            impl<T> Send for Vec<T> where T: Send { }
            impl !Send for i32 { }
        }

        goal {
            i32: Send
        } yields {
            "No possible solution"
        }

        goal {
            f32: Send
        } yields {
            "Unique"
        }

        goal {
            Vec<i32>: Send
        } yields {
            "No possible solution"
        }

        goal {
            Vec<f32>: Send
        } yields {
            "Unique"
        }

        goal {
            forall<T> {
                Vec<T>: Send
            }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn partial_overlap_2() {
    test! {
        program {
            trait Marker<T> {}
            trait Foo {}
            trait Bar {}

            struct i32 {}
            struct u32 {}

            impl<T> Marker<i32> for T where T: Foo {}
            impl<T> Marker<u32> for T where T: Bar {}
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) {
                    exists<A> { T: Marker<A> }
                }
            }
        } yields {
            "Ambiguous"
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) {
                    T: Marker<u32>
                }
            }
        } yields {
            "Unique"
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) {
                    T: Marker<i32>
                }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn partial_overlap_3() {
    test! {
        program {
            #[marker] trait Marker {}
            trait Foo {}
            trait Bar {}

            impl<T> Marker for T where T: Foo {}
            impl<T> Marker for T where T: Bar {}

            struct i32 {}
            impl Foo for i32 {}
            impl Bar for i32 {}
        }

        goal {
            forall<T> {
                if (T: Foo; T: Bar) { T: Marker }
            }
        } yields {
            "Unique"
        }

        goal {
            i32: Marker
        } yields {
            "Unique"
        }
    }
}

#[test]
fn inscope() {
    test! {
        program {
            trait Foo { }
        }

        goal {
            InScope(Foo)
        } yields {
            "No possible solution"
        }

        goal {
            if (InScope(Foo)) {
                InScope(Foo)
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
#[test]
fn overflow_universe() {
    test! {
        program {
            struct Foo { }

            trait Bar { }

            // When asked to solve X: Bar, we will produce a
            // requirement to solve !1_0: Bar. And then when asked to
            // solve that, we'll produce a requirement to solve !1_1:
            // Bar.  And so forth.
            forall<X> { X: Bar if forall<Y> { Y: Bar } }
        }

        goal {
            Foo: Bar
        } yields {
            // The internal universe canonicalization in the on-demand/recursive
            // solver means that when we are asked to solve (e.g.)
            // `!1_1: Bar`, we rewrite that to `!1_0: Bar`, identifying a
            // cycle.
            "No possible solution"
        }
    }
}

#[test]
fn projection_from_env_a() {
    test! {
        program {
            trait Sized { }

            struct Slice<T> where T: Sized { }
            impl<T> Sized for Slice<T> { }

            trait SliceExt
            {
                type Item;
            }

            impl<T> SliceExt for Slice<T>
            {
                type Item = T;
            }
        }

        goal {
            forall<T> {
                if (
                    <Slice<T> as SliceExt>::Item: Sized
                ) {
                    T: Sized
                }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn gat_unify_with_implied_wc() {
    test! {
        program {
            struct Slice<T> { }

            trait Cast<T> { }
            trait CastingIter<T> {
                type Item<U>: Cast<U> where T: Cast<U>;
            }

            impl<T> CastingIter<T> for Slice<T> {
                type Item<U> = Castable<T, U>;
            }

            struct Castable<T, U> { }
            impl<T, U> Cast<U> for Castable<T, U> { }
        }

        goal {
            forall<T, U, V> {
                if (
                    FromEnv(<Slice<T> as CastingIter<T>>::Item<U>)
                ) {
                    T: Cast<U>
                }
            }
        } yields {
            "Unique"
        }

        goal {
            forall<T, U, V> {
                T: Cast<U>
            }
        } yields {
            "No possible solution"
        }
    }
}

// This variant of the above test used to be achingly slow on SLG
// solvers, before the "trivial answer" green cut was introduced.
//
// The problem was that we wound up enumerating a goal like
//
//     <?0 as SliceExt>::Item = !1_0
//
// which meant "find me the types that normalize to `!1_0`". We had no
// problem finding these types, but after the first such type, we had
// the only unique answer we would ever find, and we wanted to reach
// the point where we could say "no more answers", so we kept
// requesting more answers.
#[test]
fn projection_from_env_slow() {
    test! {
        program {
            trait Clone { }
            trait Sized { }

            struct Slice<T> where T: Sized { }
            impl<T> Sized for Slice<T> { }

            struct u32 { }
            impl Clone for u32 { }
            impl Sized for u32 { }

            trait SliceExt
                where <Self as SliceExt>::Item: Clone
            {
                type Item;
            }

            impl<T> SliceExt for Slice<T>
                where T: Clone
            {
                type Item = T;
            }
        }

        goal {
            forall<T> {
                if (
                    <Slice<T> as SliceExt>::Item: Clone;
                    <Slice<T> as SliceExt>::Item: Sized;
                    T: Clone
                ) {
                    T: Sized
                }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn clauses_in_if_goals() {
    test! {
        program {
            trait Foo { }
            struct Vec<T> { }
            struct i32 { }
        }

        goal {
            if (forall<T> { T: Foo }) {
                forall<T> { T: Foo }
            }
        } yields {
            "Unique"
        }

        goal {
            forall<T> {
                if (Vec<T>: Foo :- T: Foo) {
                    if (T: Foo) {
                        Vec<T>: Foo
                    }
                }
            }
        } yields {
            "Unique"
        }

        goal {
            if (forall<T> { Vec<T>: Foo :- T: Foo }) {
                if (i32: Foo) {
                    Vec<i32>: Foo
                }
            }
        } yields {
            "Unique"
        }

        goal {
            if (forall<T> { Vec<T>: Foo :- T: Foo }) {
                Vec<i32>: Foo
            }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn quantified_types() {
    test! {
        program {
            trait Foo { }
            struct fn<'a> { }
            struct fn2<'a, 'b> { }
            impl Foo for for<'a> fn<'a> { }
        }

        goal {
            for<'a> fn<'a>: Foo
        } yields {
            "Unique"
        }

        goal {
            for<'a, 'b> fn2<'a, 'b> = for<'b, 'a> fn2<'a, 'b>
        } yields {
            "Unique"
        }

        goal {
            forall<'a> { fn<'a>: Foo }
        } yields {
            // Lifetime constraints are unsatisfiable
            "Unique; substitution [], \
            lifetime constraints [InEnvironment { environment: Env([]), goal: '!2_0 == '!1_0 }]"
        }
    }
}

#[test]
fn higher_ranked_implied_bounds() {
    test! {
        program {
            trait Foo<'a> { }
            trait Bar where forall<'a> Self: Foo<'a> { }
        }

        goal {
            forall<T> {
                if (T: Bar) {
                    forall<'a> {
                        T: Foo<'a>
                    }
                }
            }
        } yields {
            "Unique"
        }
    }

    test! {
        program {
            trait Foo<T> { }
            trait Bar where forall<T> Self: Foo<T> { }
        }

        goal {
            forall<T> {
                if (T: Bar) {
                    forall<U> {
                        T: Foo<U>
                    }
                }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn recursive_where_clause_on_type() {
    test! {
        program {
            trait Bar { }
            trait Foo where Self: Bar { }

            struct S where S: Foo { }

            impl Foo for S { }
        }

        goal {
            WellFormed(S)
        } yields {
            "No possible solution"
        }
    }
}
