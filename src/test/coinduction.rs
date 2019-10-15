//! Tests targeting coinduction specifically

use super::*;

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
