//! Tests related to the implied bounds rules.

use super::*;

#[test]
fn implied_bounds() {
    test! {
        program {
            trait Clone { }
            trait Iterator where Self: Clone { type Item; }
            struct Struct { }
        }

        goal {
            forall<T> {
                if (T: Iterator<Item = Struct>) {
                    T: Clone
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn gat_implied_bounds() {
    test! {
        program {
            trait Clone { }
            trait Foo { type Item<T>: Clone; }
            struct Struct { }
        }

        goal {
            forall<T, U, V> {
                if (T: Foo<Item<U> = V>) {
                    V: Clone
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }

    test! {
        program {
            trait Clone { }
            trait Foo { type Item<T>; }
            struct Struct { }
        }

        goal {
            forall<T, U, V> {
                if (T: Foo<Item<U> = V>) {
                    // Without the bound Item<T>: Clone, there is no way to infer this.
                    V: Clone
                }
            }
        } yields {
            expect![["No possible solution"]]
        }
    }

    test! {
        program {
            trait Fn<T> { }
            struct Ref<'a, T> { }
            trait Sized { }

            trait Foo {
                type Item<T>: forall<'a> Fn<Ref<'a, T>> + Sized;
            }
        }

        goal {
            forall<Type> {
                if (Type: Foo) {
                    forall<'a, T> {
                        <Type as Foo>::Item<T>: Fn<Ref<'a, T>>
                    }
                }
            }
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn implied_from_env() {
    test! {
        program {
            trait Clone { }
            trait Foo<U> { type Item<V>; }
        }

        goal {
            forall<T, U, V> {
                if (FromEnv(<T as Foo<U>>::Item<V>)) {
                    FromEnv(T: Foo<U>)
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T, U, V> {
                if (FromEnv(<T as Foo<U>>::Item<V>)) {
                    FromEnv(T: Clone)
                }
            }
        } yields {
            expect![["No possible solution"]]
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
            expect![["Unique"]]
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
            expect![["Unique"]]
        }
    }
}
