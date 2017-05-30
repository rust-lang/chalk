use chalk_parse;
use errors::*;
use ir;
use lower::*;
use solve::solver::Solver;
use std::sync::Arc;

fn parse_and_lower_program(text: &str) -> Result<ir::Program> {
    chalk_parse::parse_program(text)?.lower()
}

fn parse_and_lower_goal(program: &ir::Program, text: &str) -> Result<Box<ir::Goal>> {
    chalk_parse::parse_goal(text)?.lower(program)
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
    let env = Arc::new(program.environment());
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

            let mut solver = Solver::new(&env, overflow_depth);
            let result = match solver.solve_goal(*goal) {
                Ok(v) => format!("{}", v),
                Err(e) => format!("No possible solution: {}", e),
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
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Foo: Clone
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Bar: Clone
        } yields {
            "No possible solution"
        }

        goal {
            Vec<Bar>: Clone
        } yields {
            "No possible solution"
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
            "Ambiguous; no inference guidance"
        }

        goal {
            exists<A> { A: Map<Bar> }
        } yields {
            "Unique; substitution [?0 := Foo], lifetime constraints []"
        }

        goal {
            exists<A> { Foo: Map<A> }
        } yields {
            "Unique; substitution [?0 := Bar], lifetime constraints []"
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
            "No possible solution"
        }

        // If we assume `T: Marker`, then obviously `T: Marker`.
        goal {
            forall<T> { if (T: Marker) { T: Marker } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // We don't have know to anything about `T` to know that
        // `Vec<T>: Marker`.
        goal {
            forall<T> { Vec<T>: Marker }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // Here, we don't know that `T: Clone`, so we can't prove that
        // `Vec<T>: Clone`.
        goal {
            forall<T> { Vec<T>: Clone }
        } yields {
            "No possible solution"
        }

        // Here, we do know that `T: Clone`, so we can.
        goal {
            forall<T> {
                if (T: Clone) {
                    Vec<T>: Clone
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
            "Unique; substitution [?0 := u8], lifetime constraints []"
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
            "No possible solution"
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
            "Ambiguous; definite substitution [?0 := S<S<S<S<?0>>>>]"
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
            "Unique; substitution [?0 := !1], lifetime constraints []"
        }

        goal {
            forall<T> {
                Vec<T>: Iterator<Item = T>
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
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
            "Unique; substitution [?0 := u32], lifetime constraints []"
        }

        // TODO: re-enable this once normalization fallback is reimplemented

        // goal {
        //     forall<T> {
        //         if (T: Iterator) {
        //             exists<U> {
        //                 T: Iterator<Item = U>
        //             }
        //         }
        //     }
        // } yields {
        //     "Solution {
        //         successful: Yes,
        //         refined_goal: Query {
        //             value: Constrained {
        //                 value: [
        //                     <!1 as Iterator>::Item ==> (Iterator::Item)<!1>
        //                 ],
        //                 constraints: []
        //             },
        //             binders: []
        //         }
        //     }"
        // }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
// TODO: re-enable once normalization fallback is reimplemented
//#[test]
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
                refined_goal: Query {
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
            "Unique; substitution [],
                     lifetime constraints [
                       (Env(U2, []) |- LifetimeEq('!2, '!1))
                     ]"
        }

        goal {
            forall<'a> {
                exists<'b> {
                    Ref<'a, Unit>: Eq<Ref<'b, Unit>>
                }
            }
        } yields {
            "Unique; substitution ['?0 := '!1], lifetime constraints []"
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
            for<'a, 'b> Ref<'a, Ref<'b, Unit>>: Eq<for<'c, 'd> Ref<'c, Ref<'d, Unit>>>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            // Note: this equality is false, but we get back successful;
            // this is because the region constraints are unsolvable.
            //
            // Note that `?0` (in universe 2) must be equal to both
            // `!1` and `!2`, which of course it cannot be.
            for<'a, 'b> Ref<'a, Ref<'b, Ref<'a, Unit>>>: Eq<
                for<'c, 'd> Ref<'c, Ref<'d, Ref<'d, Unit>>>>
        } yields {
            "Unique; substitution [], lifetime constraints [
                 (Env(U2, []) |- LifetimeEq('!2, '!1))
             ]"
        }
    }
}

/// Demonstrates that, given the expected value of the associated
/// type, we can use that to narrow down the relevant impls.
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
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn elaborate_eq() {
    test! {
        program {
            trait PartialEq { }
            trait Eq where Self: PartialEq { }
        }

        goal {
            forall<T> {
                if (T: Eq) {
                    T: PartialEq
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn elaborate_transitive() {
    test! {
        program {
            trait PartialEq { }
            trait Eq where Self: PartialEq { }
            trait StrictEq where Self: Eq { }
        }

        goal {
            forall<T> {
                if (T: StrictEq) {
                    T: PartialEq
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn elaborate_normalize() {
    test! {
        program {
            trait Eq { }

            trait Item where <Self as Item>::Out: Eq {
                type Out;
            }
        }

        goal {
            forall<T, U> {
                if (T: Item<Out = U>) {
                    U: Eq
                }
            }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn atc1() {
    test! {
        program {
            struct Vec<T> { }

            trait Iterable {
                type Iter<'a>;
            }

            impl<T> Iterable for Vec<T> {
                type Iter<'a> = Iter<'a, T>;
            }

            trait Iterator {
                type Item;
            }

            struct Iter<'a, T> { }
            struct Ref<'a, T> { }

            impl<'a, T> Iterator for Iter<'a, T> {
                type Item = Ref<'a, T>;
            }
        }

        goal {
            forall<T> {
                forall<'a> {
                    exists<U> {
                        Vec<T>: Iterable<Iter<'a> = U>
                    }
                }
            }
        } yields {
            "Unique; substitution [?0 := Iter<'!2, !1>], lifetime constraints []"
        }
    }
}

#[test]
fn struct_wf() {
    test! {
        program {
            struct Foo<T> where T: Eq { }
            struct Bar { }
            struct Baz { }

            trait Eq { }

            impl Eq for Baz { }
            impl<T> Eq for Foo<T> where T: Eq { }
        }

        goal {
            WellFormed(Foo<Bar>)
        } yields {
            "No possible solution"
        }

        goal {
            WellFormed(Foo<Baz>)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed(Foo<Foo<Baz>>)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

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
fn trait_wf() {
    test! {
        program {
            struct Vec<T> where T: Sized { }
            struct Slice<T> where T: Sized { }
            struct Int { }

            trait Sized { }
            trait Eq<T> { }
            trait Ord<T> where Self: Eq<T> { }

            impl<T> Sized for Vec<T> where T: Sized { }
            impl Sized for Int { }

            impl Eq<Int> for Int { }
            impl<T> Eq<Vec<T>> for Vec<T> where T: Eq<T> { }

            impl Ord<Int> for Int { }
            impl<T> Ord<Vec<T>> for Vec<T> where T: Ord<T> { }
        }

        goal {
            WellFormed(Slice<Int>)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Slice<Int>: Sized
        } yields {
            "No possible solution"
        }

        goal {
            WellFormed(Slice<Int>: Sized)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WellFormed(Slice<Int>: Eq<Slice<Int>>)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            Slice<Int>: Eq<Slice<Int>>
        } yields {
            "No possible solution"
        }

        // not WF because previous equation doesn't hold
        goal {
            WellFormed(Slice<Int>: Ord<Slice<Int>>)
        } yields {
            "No possible solution"
        }

        goal {
            Vec<Int>: Eq<Vec<Int>>
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        // WF because previous equation does hold
        goal {
            WellFormed(Vec<Int>: Ord<Vec<Int>>)
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}

#[test]
fn normalize_under_binder() {
    test! {
        program {
            struct Ref<'a, T> { }
            struct I32 { }

            trait Deref<'a> {
                type Item;
            }

            trait Id<'a> {
                type Item;
            }

            impl<'a, T> Deref<'a> for Ref<'a, T> {
                type Item = T;
            }

            impl<'a, T> Id<'a> for Ref<'a, T> {
                type Item = Ref<'a, T>;
            }
        }

        goal {
            exists<U> {
                forall<'a> {
                    Ref<'a, I32>: Deref<'a, Item = U>
                }
            }
        } yields {
            "Unique; substitution [?0 := I32], lifetime constraints []"
        }

        goal {
            forall<'a> {
                exists<U> {
                    Ref<'a, I32>: Id<'a, Item = U>
                }
            }
        } yields {
            "Unique; substitution [?0 := Ref<'!1, I32>], lifetime constraints []"
        }

        goal {
            exists<U> {
                forall<'a> {
                    Ref<'a, I32>: Id<'a, Item = U>
                }
            }
        } yields {
            "Unique; substitution [?0 := Ref<'?0, I32>], lifetime constraints [
                 (Env(U0, []) |- LifetimeEq('?0, '!1))
             ]"
        }
    }
}

#[test]
fn unify_quantified_lifetimes() {
    test! {
        program {
        }

        // Check that `'a` (here, `'?0`) is not unified
        // with `'!1`, because they belong to incompatible
        // universes.
        goal {
            exists<'a> {
                forall<'b> {
                    'a = 'b
                }
            }
        } yields {
            "Unique; substitution ['?0 := '?0], lifetime constraints [
                 (Env(U1, []) |- LifetimeEq('?0, '!1))
             ]"
        }

        // Similar to the previous test, but indirect.
        goal {
            exists<'a> {
                forall<'b> {
                    exists<'c> {
                        'a = 'c,
                        'c = 'b
                    }
                }
            }
        } yields {
            "Unique; substitution ['?0 := '?0, '?1 := '!1], lifetime constraints [
                 (Env(U1, []) |- LifetimeEq('?0, '!1))
             ]"
        }
    }
}

#[test]
fn equality_binder() {
    test! {
        program {
            struct Ref<'a, T> { }
        }

        // Check that `'a` (here, `'?0`) is not unified
        // with `'!1`, because they belong to incompatible
        // universes.
        goal {
            forall<T> {
                exists<'a> {
                    for<'c> Ref<'c, T> = Ref<'a, T>
                }
            }
        } yields {
            "Unique; substitution ['?0 := '?0], lifetime constraints [
                 (Env(U2, []) |- LifetimeEq('!2, '?0))
             ]"
        }
    }
}

#[test]
fn mixed_indices_unify() {
    test! {
        program {
            struct Ref<'a, T> { }
        }

        goal {
            exists<T> {
                exists<'a> {
                    exists<U> {
                        Ref<'a, T> = Ref<'a, U>
                    }
                }
            }
        } yields {
            "Unique; substitution [?0 := ?0, ?1 := ?0, '?0 := '?1], lifetime constraints []"
        }
    }
}

#[test]
fn mixed_indices_match_program() {
    test! {
        program {
            struct S { }
            struct Bar<'a, T, U> { }
            trait Foo {}
            impl<'a> Foo for Bar<'a, S, S> {}
        }

        goal {
            exists<T> {
                exists<'a> {
                    exists<U> {
                        Bar<'a, T, U>: Foo
                    }
                }
            }
        } yields {
            "Unique; substitution [?0 := S, ?1 := S, '?0 := '?0], lifetime constraints []"
        }
    }
}

// TODO: re-enable once normalization fallback is reimplemented
//#[test]
fn mixed_indices_normalize_application() {
    test! {
        program {
            struct Ref<'a, T> { }
            trait Foo {
                type T;
            }
        }

        goal {
            exists<T> {
                exists<'a> {
                    exists<U> {
                        <Ref<'a, T> as Foo>::T = U
                    }
                }
            }
        } yields {
            "Solution { successful: Yes"
        }
    }
}

#[test]
// Test that we properly detect failure even if there are applicable impls at
// the top level, if we can't find anything to fill in those impls with
fn test_deep_failure() {
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
fn test_deep_success() {
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
fn test_definite_guidance() {
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
            "Ambiguous; definite substitution [?0 := Foo<?0>]"
        }
    }
}

#[test]
fn test_suggested_subst() {
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
            "Ambiguous; suggested substitution [?0 := bool]"
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
            "Ambiguous; suggested substitution [?0 := bool]"
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
fn test_simple_negation() {
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
            "Ambiguous"
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
    }
}

#[test]
fn test_deep_negation() {
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
fn test_negation_quantifiers() {
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
            "Ambig"
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
            "Ambig"
        }
    }
}
