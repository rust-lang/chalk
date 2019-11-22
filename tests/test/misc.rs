//! Tests that don't fit a single category

use super::*;

// Regression test for rust-lang/chalk#111
#[test]
fn futures_ambiguity() {
    test! {
        program {
            struct Result<T, E> { }

            trait Future {
                type Output;
            }

            trait FutureResult
                where
                Self: Future<Output = Result<
                    <Self as FutureResult>::Item,
                    <Self as FutureResult>::Error
                >>
            {
                type Item;
                type Error;
            }

            impl<T, I, E> FutureResult for T
                where
                T: Future<Output = Result<I, E>>
            {
                type Item = I;
                type Error = E;
            }
        }

        goal {
            forall<T> { if (T: FutureResult) { exists<I, E> { T: Future<Output = Result<I, E>> } } }
        } yields {
            r"Unique; substitution [?0 := (FutureResult::Item)<!1_0>, ?1 := (FutureResult::Error)<!1_0>], lifetime constraints []"
        }
    }
}
