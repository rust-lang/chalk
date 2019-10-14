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

mod cycle;
mod implied_bounds;
mod impls;
mod projection;
mod unify;
mod wf_goals;

#[test]
fn generic_trait() {
    test! {
        program {
            struct Int { }
            struct Uint { }

            trait Eq<T> { }

            impl Eq<Int> for Int { }
            impl Eq<Uint> for Uint { }
        }

        goal {
            Int: Eq<Int>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Uint: Eq<Uint>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Int: Eq<Uint>
        } yields {
            "No possible solution"
        }
    }
}

#[test]
// Test that we properly detect failure even if there are applicable impls at
// the top level, if we can't find anything to fill in those impls with
fn deep_failure() {
    test! {
        program {
            struct Foo<T> {}
            trait Bar {}
            trait Baz {}

            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            exists<T> { T: Baz }
        } yields {
            "No possible solution"
        }

        goal {
            exists<T> { Foo<T>: Bar }
        } yields {
            "No possible solution"
        }
    }
}

#[test]
// Test that we infer a unique solution even if it requires multiple levels of
// search to do so
fn deep_success() {
    test! {
        program {
            struct Foo<T> {}
            struct ImplsBaz {}
            trait Bar {}
            trait Baz {}

            impl Baz for ImplsBaz {}
            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            exists<T> { Foo<T>: Bar }
        } yields {
            "Unique; substitution [?0 := ImplsBaz]"
        }
    }
}

#[test]
fn definite_guidance() {
    test! {
        program {
            trait Display {}
            trait Debug {}
            struct Foo<T> {}
            struct Bar {}
            struct Baz {}

            impl Display for Bar {}
            impl Display for Baz {}

            impl<T> Debug for Foo<T> where T: Display {}
        }

        goal {
            exists<T> {
                T: Debug
            }
        } yields {
            "Ambiguous; definite substitution for<?U0> { [?0 := Foo<^0>] }"
        }
    }
}

#[test]
fn suggested_subst() {
    test! {
        program {
            trait SomeTrait<A> {}
            struct Foo {}
            struct Bar {}
            struct i32 {}
            struct bool {}
            impl SomeTrait<i32> for Foo {}
            impl SomeTrait<bool> for Bar {}
            impl SomeTrait<i32> for Bar {}
        }

        goal {
            exists<T> {
                Foo: SomeTrait<T>
            }
        } yields {
            "Unique; substitution [?0 := i32]"
        }

        goal {
            exists<T> {
                if (i32: SomeTrait<bool>) {
                    i32: SomeTrait<T>
                }
            }
        } yields {
            "Unique; substitution [?0 := bool]"
        }

        goal {
            exists<T> {
                if (i32: SomeTrait<bool>) {
                    Foo: SomeTrait<T>
                }
            }
        } yields {
            "Unique; substitution [?0 := i32]"
        }

        goal {
            exists<T> {
                if (Foo: SomeTrait<i32>) {
                    Foo: SomeTrait<T>
                }
            }
        } yields {
            "Unique; substitution [?0 := i32]"
        }

        goal {
            exists<T> {
                if (Foo: SomeTrait<bool>) {
                    Foo: SomeTrait<T>
                }
            }
        } yields {
            // FIXME: we need to rework the "favor environment" heuristic.
            // Should be: "Ambiguous; suggested substitution [?0 := bool]"
            "Ambiguous; no inference guidance"
        }

        goal {
            exists<T> {
                if (Foo: SomeTrait<bool>) {
                    if (Foo: SomeTrait<i32>) {
                        Foo: SomeTrait<T>
                    }
                }
            }
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            exists<T> {
                Bar: SomeTrait<T>
            }
        } yields {
            "Ambiguous; no inference guidance"
        }

        goal {
            exists<T> {
                if (Bar: SomeTrait<bool>) {
                    Bar: SomeTrait<T>
                }
            }
        } yields {
            // FIXME: same as above, should be: "Ambiguous; suggested substitution [?0 := bool]"
            "Ambiguous; no inference guidance"
        }

        goal {
            exists<T> {
                if (Bar: SomeTrait<bool>) {
                    if (Bar: SomeTrait<i32>) {
                        Bar: SomeTrait<T>
                    }
                }
            }
        } yields {
            "Ambiguous; no inference guidance"
        }
    }
}

#[test]
fn simple_negation() {
    test! {
        program {
            struct i32 {}
            trait Foo {}
        }

        goal {
            not { i32: Foo }
        } yields {
            "Unique"
        }

        goal {
            not {
                not { i32: Foo }
            }
        } yields {
            "No"
        }

        goal {
            not {
                not {
                    not { i32: Foo }
                }
            }
        } yields {
            "Unique"
        }

        goal {
            exists<T> {
                not { T: Foo }
            }
        } yields {
            "Ambig"
        }

        goal {
            forall<T> {
                not { T: Foo }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                exists<T> { T: Foo }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                forall<T> { T: Foo }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn deep_negation() {
    test! {
        program {
            struct Foo<T> {}
            trait Bar {}
            trait Baz {}

            impl<T> Bar for Foo<T> where T: Baz {}
        }

        goal {
            not {
                exists<T> { T: Baz }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                exists<T> { Foo<T>: Bar }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn negation_quantifiers() {
    test! {
        program {
            struct i32 {}
            struct u32 {}
        }

        goal {
            not {
                forall<T, U> {
                    T = U
                }
            }
        } yields {
            "Unique"
        }

        goal {
            not {
                exists<T, U> {
                    T = U
                }
            }
        } yields {
            "No"
        }

        goal {
            forall<T, U> {
                not {
                    T = U
                }
            }
        } yields {
            "No"
        }
    }
}

#[test]
fn negation_free_vars() {
    test! {
        program {
            struct Vec<T> {}
            struct i32 {}
            struct u32 {}
            trait Foo {}
            impl Foo for Vec<u32> {}
        }

        goal {
            exists<T> {
                not { Vec<T>: Foo }
            }
        } yields {
            "Ambig"
        }
    }
}

#[test]
fn where_clause_trumps() {
    test! {
        program {
            struct Foo { }

            trait Marker { }
            impl Marker for Foo { }
        }

        goal {
            forall<T> {
                if (T: Marker) {
                    T: Marker
                }
            }
        } yields {
            "Unique"
        }
    }
}

#[test]
fn inapplicable_assumption_does_not_shadow() {
    test! {
        program {
            struct i32 { }
            struct u32 { }

            trait Foo<T> { }

            impl<T> Foo<i32> for T { }
        }

        goal {
            forall<T> {
                exists<U> {
                    if (i32: Foo<T>) {
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
fn coinductive_semantics() {
    test! {
        program {
            #[auto] trait Send { }

            struct i32 { }

            struct Ptr<T> { }
            impl<T> Send for Ptr<T> where T: Send { }

            struct List<T> {
                data: T,
                next: Ptr<List<T>>
            }
        }

        goal {
            forall<T> {
                List<T>: Send
            }
        } yields {
            "No possible solution"
        }
        goal {
            forall<T> {
                if (T: Send) {
                    List<T>: Send
                }
            }
        } yields {
            "Unique"
        }

        goal {
            List<i32>: Send
        } yields {
            "Unique"
        }

        goal {
            exists<T> {
                T: Send
            }
        } yields {
            "Ambiguous"
        }
    }
}

#[test]
fn mixed_semantics() {
    test! {
        program {
            #[auto] trait Send { }
            trait Foo { }

            struct Bar { }

            impl Send for Bar where Bar: Foo { }
            impl Foo for Bar where Bar: Send { }
        }

        // We have a cycle `(T: Send) :- (T: Foo) :- (T: Send)` with a non-coinductive
        // inner component `T: Foo` so we reject it.
        goal {
            Bar: Send
        } yields {
            "No possible solution"
        }

        goal {
            Bar: Foo
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

#[test]
fn local_and_upstream_types() {
    test! {
        program {
            #[upstream] struct Upstream { }
            struct Local { }
        }

        goal { IsLocal(Upstream) } yields { "No possible solution" }
        goal { IsUpstream(Upstream) } yields { "Unique" }

        goal { IsLocal(Local) } yields { "Unique" }
        goal { IsUpstream(Local) } yields { "No possible solution" }
    }

    test! {
        program {
            trait Clone { }
            #[upstream] struct Upstream<T> where T: Clone { }
            struct Local<T> where T: Clone { }

            #[upstream] struct Upstream2 { }
            struct Internal2 { }
        }

        goal { forall<T> { IsLocal(Upstream<T>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Upstream<T>) } } yields { "Unique" }

        goal { forall<T> { IsLocal(Local<T>) } } yields { "Unique" }
        goal { forall<T> { IsUpstream(Local<T>) } } yields { "No possible solution" }
    }
}

#[test]
fn is_fully_visible() {
    // Should be visible regardless of local, fundamental, or upstream
    test! {
        program {
            #[upstream] struct Upstream { }
            struct Local { }

            #[upstream]
            #[fundamental]
            struct Box<T> { }
        }

        goal { IsFullyVisible(Upstream) } yields { "Unique" }
        goal { IsFullyVisible(Local) } yields { "Unique" }
        goal { IsFullyVisible(Box<Local>) } yields { "Unique" }
        goal { IsFullyVisible(Box<Upstream>) } yields { "Unique" }
    }

    // Should be visible regardless of local, fundamental, or upstream
    test! {
        program {
            #[upstream] struct Upstream { }
            struct Local { }

            #[upstream] struct Upstream2<T> { }
            struct Local2<T> { }

            #[upstream]
            #[fundamental]
            struct Box<T> { }
        }

        // Unknown type parameters are not fully visible
        goal { forall<T> { IsFullyVisible(Box<T>) } } yields { "No possible solution" }
        goal { forall<T> { IsFullyVisible(Upstream2<T>) } } yields { "No possible solution" }
        goal { forall<T> { IsFullyVisible(Local2<T>) } } yields { "No possible solution" }

        // Without any unknown type parameters, local and upstream should not matter
        goal { forall<T> { IsFullyVisible(Upstream2<Upstream>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Upstream2<Local>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Local2<Upstream>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Local2<Local>) } } yields { "Unique" }

        // Fundamental anywhere should not change the outcome
        goal { forall<T> { IsFullyVisible(Box<Upstream2<Upstream>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Box<Upstream2<Local>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Box<Local2<Upstream>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Box<Local2<Local>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Upstream2<Box<Upstream>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Upstream2<Box<Local>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Local2<Box<Upstream>>) } } yields { "Unique" }
        goal { forall<T> { IsFullyVisible(Local2<Box<Local>>) } } yields { "Unique" }
    }
}

#[test]
fn fundamental_types() {
    // NOTE: These tests need to have both Local and Upstream structs since chalk will attempt
    // to enumerate all of them.

    // This first test is a sanity check to make sure `Box` isn't a special case.
    // By testing this, we ensure that adding the #[fundamental] attribute does in fact
    // change behaviour
    test! {
        program {
            #[upstream] struct Box<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        // Without fundamental, Box should behave like a regular upstream type
        goal { forall<T> { not { IsLocal(Box<T>) } } } yields { "Unique" }
        goal { forall<T> { IsLocal(Box<T>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Box<T>) } } yields { "Unique" }

        // Without fundamental, Box is upstream regardless of its inner type
        goal { IsLocal(Box<Upstream>) } yields { "No possible solution" }
        goal { IsLocal(Box<Local>) } yields { "No possible solution" }
        goal { IsUpstream(Box<Upstream>) } yields { "Unique" }
        goal { IsUpstream(Box<Local>) } yields { "Unique" }
    }

    test! {
        program {
            #[upstream]
            #[fundamental]
            struct Box<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        // With fundamental, Box can be local for certain types, so there is no unique solution
        // anymore for any of these
        goal { forall<T> { not { IsLocal(Box<T>) } } } yields { "Ambiguous" }
        goal { forall<T> { IsLocal(Box<T>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Box<T>) } } yields { "No possible solution" }

        // With fundamental, some of these yield different results -- no longer depends on Box
        // itself
        goal { IsLocal(Box<Upstream>) } yields { "No possible solution" }
        goal { IsLocal(Box<Local>) } yields { "Unique" }
        goal { IsUpstream(Box<Upstream>) } yields { "Unique" }
        goal { IsUpstream(Box<Local>) } yields { "No possible solution" }
    }

    test! {
        program {
            #[upstream]
            #[fundamental]
            struct Box<T> { }

            trait Clone { }
            #[upstream] struct Upstream<T> where T: Clone { }
            struct Local<T> where T: Clone { }

            #[upstream] struct Upstream2 { }
            struct Internal2 { }
        }

        // Upstream is upstream no matter what, so this should not be local for any T
        goal { forall<T> { IsLocal(Box<Upstream<T>>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Box<Upstream<T>>) } } yields { "Unique" }

        // A fundamental type inside an upstream type should not make a difference (i.e. the rules
        // for the outer, non-fundamental type should apply)
        goal { forall<T> { IsLocal(Upstream<Box<T>>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Upstream<Box<T>>) } } yields { "Unique" }

        // Make sure internal types within an upstream type do not make a difference
        goal { forall<T> { IsLocal(Box<Upstream<Local<T>>>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Box<Upstream<Local<T>>>) } } yields { "Unique" }

        // Local is local no matter what, so this should be local for any T
        goal { forall<T> { IsLocal(Box<Local<T>>) } } yields { "Unique" }
        goal { forall<T> { IsUpstream(Box<Local<T>>) } } yields { "No possible solution" }

        // A fundamental type inside an internal type should not make a difference
        goal { forall<T> { IsLocal(Local<Box<T>>) } } yields { "Unique" }
        goal { forall<T> { IsUpstream(Local<Box<T>>) } } yields { "No possible solution" }

        // Make sure upstream types within an internal type and vice versa do not make a difference
        goal { forall<T> { IsLocal(Box<Local<Upstream<T>>>) } } yields { "Unique" }
        goal { forall<T> { IsUpstream(Box<Upstream<Local<T>>>) } } yields { "Unique" }
    }

    // Nested fundamental types should still be local if they can be recursively proven to be local
    test! {
        program {
            #[upstream]
            #[fundamental]
            struct Box<T> { }
            // This type represents &T which is also fundamental
            #[upstream]
            #[fundamental]
            struct Ref<T> { }

            trait Clone { }
            #[upstream] struct Upstream<T> where T: Clone { }
            struct Local<T> where T: Clone { }

            #[upstream] struct Upstream2 { }
            struct Internal2 { }
        }

        goal { forall<T> { IsLocal(Ref<Box<T>>) } } yields { "No possible solution" }
        goal { forall<T> { IsUpstream(Ref<Box<T>>) } } yields { "No possible solution" }

        goal { IsLocal(Ref<Box<Upstream2>>) } yields { "No possible solution" }
        goal { IsUpstream(Ref<Box<Upstream2>>) } yields { "Unique" }

        goal { IsLocal(Ref<Box<Internal2>>) } yields { "Unique" }
        goal { IsUpstream(Ref<Box<Internal2>>) } yields { "No possible solution" }
    }

    // If a type is not upstream, it is always local regardless of its parameters or #[fundamental]
    test! {
        program {
            // if we were compiling std, Box would never be upstream
            #[fundamental]
            struct Box<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        goal { forall<T> { IsLocal(Box<T>) } } yields { "Unique" }
        goal { IsLocal(Box<Upstream>) } yields { "Unique" }
        goal { IsLocal(Box<Local>) } yields { "Unique" }
    }
}

#[test]
fn local_impl_allowed_for_traits() {
    test! {
        program {
            trait LocalTrait { }
            trait LocalTrait2<T> { }

            #[upstream] struct Upstream { }
            struct Local { }
        }

        // Local traits are always implementable
        goal { forall<T> { LocalImplAllowed(T: LocalTrait) } } yields { "Unique" }
        goal { LocalImplAllowed(Local: LocalTrait) } yields { "Unique" }
        goal { LocalImplAllowed(Upstream: LocalTrait) } yields { "Unique" }
        goal { forall<T> { LocalImplAllowed(T: LocalTrait2<T>) } } yields { "Unique" }
        goal { forall<T, U> { LocalImplAllowed(T: LocalTrait2<U>) } } yields { "Unique" }
        goal { forall<T> { LocalImplAllowed(Local: LocalTrait2<T>) } } yields { "Unique" }
        goal { forall<T> { LocalImplAllowed(Upstream: LocalTrait2<T>) } } yields { "Unique" }
    }

    // Single-type parameter trait refs (Self only)
    test! {
        program {
            #[upstream] trait UpstreamTrait { }

            #[upstream] struct Upstream { }
            #[upstream] struct Upstream2<T> { }
            struct Local { }
            struct Local2<T> { }
        }

        // No local type
        goal { LocalImplAllowed(Upstream: UpstreamTrait) } yields { "No possible solution" }
        goal { forall<T> { LocalImplAllowed(T: UpstreamTrait) } } yields { "No possible solution" }

        // Local type, not preceded by anything
        // Notice that the types after the first local type do not matter at all
        goal { LocalImplAllowed(Local: UpstreamTrait) } yields { "Unique" }
    }

    // Multi-type parameter trait refs (Self, T)
    test! {
        program {
            trait Clone { }
            #[upstream] trait UpstreamTrait2<T> where T: Clone { }

            #[upstream] struct Upstream { }
            #[upstream] struct Upstream2<T> { }
            struct Local { }
            struct Local2<T> { }
        }

        // No local type
        goal { forall<T> { LocalImplAllowed(T: UpstreamTrait2<T>) } } yields { "No possible solution" }
        goal { forall<T, U> { LocalImplAllowed(T: UpstreamTrait2<U>) } } yields { "No possible solution" }
        goal { forall<T> { LocalImplAllowed(Upstream: UpstreamTrait2<T>) } } yields { "No possible solution" }

        // Local type, but preceded by a type parameter
        goal { forall<T> { LocalImplAllowed(T: UpstreamTrait2<Local>) } } yields { "No possible solution" }

        // Local type, not preceded by anything
        // Notice that the types after the first local type do not matter at all
        goal { forall<T> { LocalImplAllowed(Local: UpstreamTrait2<T>) } } yields { "Unique" }
        goal { LocalImplAllowed(Local: UpstreamTrait2<Upstream>) } yields { "Unique" }
        goal { LocalImplAllowed(Local: UpstreamTrait2<Local>) } yields { "Unique" }

        // Local type, but preceded by a fully visible type (i.e. no placeholder types)
        goal { LocalImplAllowed(Upstream: UpstreamTrait2<Local>) } yields { "Unique" }
        goal { LocalImplAllowed(Upstream2<Local>: UpstreamTrait2<Local>) } yields { "Unique" }
        goal { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait2<Local>) } yields { "Unique" }

        // Type parameter covered by the local type
        goal { forall<T> { LocalImplAllowed(Upstream: UpstreamTrait2<Local2<T>>) } } yields { "Unique" }
        goal { forall<T> { LocalImplAllowed(Upstream2<Local>: UpstreamTrait2<Local2<T>>) } } yields { "Unique" }
        goal { forall<T> { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait2<Local2<T>>) } } yields { "Unique" }

        // Type parameter covered by a deeply nested upstream type
        // Notice that it does not matter that the T is wrapped in a local type because the outer
        // type is still upstream
        goal { forall<T> { LocalImplAllowed(Upstream2<Local2<T>>: UpstreamTrait2<Local2<T>>) } } yields { "No possible solution" }
        // Does not matter whether the covered type parameter is eventually covered or not by the
        // first actually local type found
        goal { forall<T, U> { LocalImplAllowed(Upstream2<Local2<T>>: UpstreamTrait2<Local2<U>>) } } yields { "No possible solution" }
    }

    test! {
        program {
            trait Clone { }
            trait Eq { }
            // Lifetime is just included to show that it does not break anything.
            // Where clauses do not change the results at all.
            #[upstream] trait UpstreamTrait<'a, T, U, V> where T: Clone, U: Eq, V: Clone, V: Eq { }
            trait InternalTrait<'a, T, U, V> where T: Clone, U: Eq, V: Clone, V: Eq { }

            #[upstream] struct Upstream { }
            #[upstream] struct Upstream2<T> { }
            struct Local { }
        }

        // Local traits can be implemented regardless of the types involved
        goal { forall<Self, 'a, T, U, V> { LocalImplAllowed(Self: InternalTrait<'a, T, U, V>) } } yields { "Unique" }

        // Upstream traits definitely cannot be implemented for all types
        goal { forall<Self, 'a, T, U, V> { LocalImplAllowed(Self: UpstreamTrait<'a, T, U, V>) } } yields { "No possible solution" }

        // No local types
        goal { forall<'a> { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait<'a, Upstream, Upstream, Upstream>) } } yields { "No possible solution" }
        goal { forall<'a> { LocalImplAllowed(Upstream2<Upstream>: UpstreamTrait<
            'a,
            Upstream2<Upstream>,
            Upstream2<Upstream2<Upstream2<Upstream>>>,
            Upstream2<Upstream2<Upstream>>
        >) } } yields { "No possible solution" }

        // Local type, not preceded by anything -- types after the first local type do not matter
        goal { forall<'a, T, U, V> { LocalImplAllowed(Local: UpstreamTrait<'a, T, U, V>) } } yields { "Unique" }
        goal { forall<'a, U, V> { LocalImplAllowed(Local: UpstreamTrait<'a, Local, U, V>) } } yields { "Unique" }
        goal { forall<'a, U, V> { LocalImplAllowed(Local: UpstreamTrait<'a, Upstream, U, V>) } } yields { "Unique" }
        goal { forall<'a> { LocalImplAllowed(Local: UpstreamTrait<'a, Upstream, Local, Local>) } } yields { "Unique" }

        // Local type preceded by a type that is not fully visible
        goal { forall<'a, T> { LocalImplAllowed(T: UpstreamTrait<'a, Upstream, Upstream, Local>) } } yields { "No possible solution" }
        goal { forall<'a, T> { LocalImplAllowed(Upstream: UpstreamTrait<'a, T, Upstream, Local>) } } yields { "No possible solution" }
        goal { forall<'a, T> { LocalImplAllowed(Upstream: UpstreamTrait<'a, Upstream, T, Local>) } } yields { "No possible solution" }

        // Once again, types after the first local do not matter
        goal { forall<'a, T> { LocalImplAllowed(Upstream: UpstreamTrait<'a, Upstream, Local, T>) } } yields { "Unique" }
    }
}
