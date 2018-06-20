#![cfg(test)]

use test_util::*;

#[test]
fn two_impls_for_same_type() {
    lowering_error! {
        program {
            trait Foo { }
            struct Bar { }
            impl Foo for Bar { }
            impl Foo for Bar { }
        }
        error_msg {
            "overlapping impls of trait \"Foo\""
        }
    }
}

#[test]
fn generic_vec_and_specific_vec() {
    lowering_success! {
        program {
            trait Foo { }
            struct Vec<T> { }
            struct Bar { }
            impl Foo for Vec<Bar> { }
            impl<T> Foo for Vec<T> { }
        }
    }
}

#[test]
fn concrete_impl_and_blanket_impl() {
    lowering_success! {
        program {
            trait Foo { }
            struct Bar { }
            impl Foo for Bar { }
            impl<T> Foo for T { }
        }
    }
}

#[test]
fn two_blanket_impls() {
    lowering_error! {
        program {
            trait Foo { }
            trait Bar { }
            trait Baz { }
            impl<T> Foo for T where T: Bar { }
            impl<T> Foo for T where T: Baz { }
            struct Quux { }
            impl Bar for Quux { }
            impl Baz for Quux { }
        }
        error_msg {
            "overlapping impls of trait \"Foo\""
        }
    }
}

#[test]
// FIXME This should be an error
// We currently assume a closed universe always, but overlaps checking should
// assume an open universe - what if a client implemented both Bar and Baz
//
// In other words, this should have the same behavior as the two_blanket_impls
// test.
fn two_blanket_impls_open_ended() {
    lowering_success! {
        program {
            trait Foo { }
            trait Bar { }
            trait Baz { }
            impl<T> Foo for T where T: Bar { }
            impl<T> Foo for T where T: Baz { }
        }
    }
}

#[test]
fn multiple_nonoverlapping_impls() {
    lowering_success! {
        program {
            trait Foo { }
            struct Bar { }
            struct Baz<T> { }
            impl Foo for Bar { }
            impl<T> Foo for Baz<T> { }
        }
    }
}

#[test]
fn local_negative_reasoning_in_coherence() {
    lowering_success! {
        program {
            trait Foo { }
            trait Bar { }
            struct Baz { }
            impl<T> Foo for T where T: Bar { }
            impl Foo for Baz { }
        }
    }
}

#[test]
fn multiple_parameters() {
    lowering_error! {
        program {
            trait Foo<T> { }
            struct Baz { }

            impl<T> Foo<Baz> for T { }
            impl<T> Foo<T> for Baz { }
        } error_msg {
            "overlapping impls of trait \"Foo\""
        }
    }
}

#[test]
fn nonoverlapping_assoc_types() {
    lowering_success! {
        program {
            trait Iterator {
                type Item;
            }
            struct Bar { }
            impl Iterator for Bar {
                type Item = Bar;
            }
            struct Baz<T> { }
            impl<T> Iterator for Baz<T> {
                type Item = Baz<T>;
            }

            trait Foo { }
            impl Foo for <Bar as Iterator>::Item { }
            impl<T> Foo for <Baz<T> as Iterator>::Item { }
        }
    }
}

#[test]
fn overlapping_assoc_types() {
    lowering_success! {
        program {
            trait Foo<T> { }

            trait Iterator { type Item; }


            struct Vec<T> { }
            impl<T> Iterator for Vec<T> { type Item = T; }

            // This impl overlaps with the one below, but specializes it.
            impl<T> Foo<<T as Iterator>::Item> for T where T: Iterator { }

            impl<A, B> Foo<A> for B { }
        }
    }
}

#[test]
fn overlapping_assoc_types_error() {
    lowering_error! {
        program {
            trait Foo<T> { }

            trait Bar { }

            trait Iterator { type Item; }


            struct Vec<T> { }
            impl<T> Iterator for Vec<T> { type Item = T; }

            struct Other { }
            impl Bar for Other { }

            // This impl overlaps with the one below, and does not
            // specialize because don't know that bar holds.
            impl<T> Foo<<T as Iterator>::Item> for T where T: Iterator { }

            impl<A, B> Foo<A> for B where A: Bar { }
        } error_msg {
            "overlapping impls of trait \"Foo\""
        }
    }
}

#[test]
fn overlapping_negative_positive_impls() {
    lowering_error! {
        program {
            trait Send { }
            struct i32 { }

            impl Send for i32 { }
            impl !Send for i32 { }
        } error_msg {
            "overlapping impls of trait \"Send\""
        }
    }
}

#[test]
fn overlapping_negative_impls() {
    lowering_success! {
        program {
            trait Send { }
            trait Foo { }
            trait Bar { }

            struct Vec<T> { }
            struct i32 { }

            impl Foo for i32 { }
            impl Bar for i32 { }

            impl<T> !Send for Vec<T> where T: Foo { }
            impl<T> !Send for Vec<T> where T: Bar { }
        }
    }
}

#[test]
fn orphan_check() {
    // These tests are largely adapted from the compile-fail coherence-*.rs tests from rustc

    lowering_error! {
        program {
            #[upstream] trait Foo { }
            #[upstream] struct Bar { }

            impl Foo for Bar { }
        } error_msg {
            "impl for trait \"Foo\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Foo { }

            impl<T> Foo for T { }
        } error_msg {
            "impl for trait \"Foo\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Foo<T> { }
            struct Bar { }

            impl<T> Foo<Bar> for T { }
        } error_msg {
            "impl for trait \"Foo\" violates the orphan rules"
        }
    }

    // Test that the `Pair` type reports an error if it contains type
    // parameters, even when they are covered by local types. This test
    // was originally intended to test the opposite, but the rules changed
    // with RFC 1023 and this became illegal.
    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Pair<T, U> { }
            struct Cover<T> { }

            impl<T> Remote for Pair<T, Cover<T>> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }
    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Pair<T, U> { }
            struct Cover<T> { }

            impl<T> Remote for Pair<Cover<T>, T> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }
    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Pair<T, U> { }
            struct Cover<T> { }

            impl<T, U> Remote for Pair<Cover<T>, U> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[auto] #[upstream] trait Send { }
            #[upstream] trait TheTrait<T> { }
            #[upstream] struct isize { }
            #[upstream] struct usize { }

            struct TheType { }

            // These impls should be fine because they contain the local type
            impl TheTrait<TheType> for isize { }
            impl TheTrait<isize> for TheType { }

            // This impl should fail because it contains only upstream type
            impl TheTrait<usize> for isize { }
        } error_msg {
            "impl for trait \"TheTrait\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[auto] #[upstream] trait Send { }
            #[upstream] struct Vec<T> { }
            #[upstream] struct isize { }

            impl !Send for Vec<isize> { }
        } error_msg {
            "impl for trait \"Send\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Pair<T, U> { }

            struct Foo { }

            impl<T> Remote for Pair<T, Foo> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Remote1<T> { }
            #[upstream] struct Pair<T, U> { }
            #[upstream] struct i32 { }

            struct Local<T> { }

            impl<T, U> Remote1<Pair<T, Local<U>>> for i32 { }
        } error_msg {
            "impl for trait \"Remote1\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Pair<T, U> { }

            struct Local<T> { }

            impl<T, U> Remote for Pair<T, Local<U>> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Vec<T> { }

            struct Local { }

            impl Remote for Vec<Local> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }

    lowering_error! {
        program {
            #[upstream] trait Remote { }
            #[upstream] struct Vec<T> { }

            struct Local<T> { }

            impl<T> Remote for Vec<Local<T>> { }
        } error_msg {
            "impl for trait \"Remote\" violates the orphan rules"
        }
    }
}
