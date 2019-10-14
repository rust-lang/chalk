//! Tests related to the implied bounds rules.

use super::*;

#[test]
fn implied_bounds() {
    test! {
        program {
            trait Clone { }
            trait Iterator where Self: Clone { type Item; }
            struct u32 { }
        }

        goal {
            forall<T> {
                if (T: Iterator<Item = u32>) {
                    T: Clone
                }
            }
        } yields {
            "Unique; substitution []"
        }
    }
}

#[test]
fn gat_implied_bounds() {
    test! {
        program {
            trait Clone { }
            trait Foo { type Item<T>: Clone; }
            struct u32 { }
        }

        goal {
            forall<T, U, V> {
                if (T: Foo<Item<U> = V>) {
                    V: Clone
                }
            }
        } yields {
            "Unique; substitution []"
        }
    }

    test! {
        program {
            trait Clone { }
            trait Foo { type Item<T>; }
            struct u32 { }
        }

        goal {
            forall<T, U, V> {
                if (T: Foo<Item<U> = V>) {
                    // Without the bound Item<T>: Clone, there is no way to infer this.
                    V: Clone
                }
            }
        } yields {
            "No possible solution"
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
            "Unique"
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
            "Unique"
        }

        goal {
            forall<T, U, V> {
                if (FromEnv(<T as Foo<U>>::Item<V>)) {
                    FromEnv(T: Clone)
                }
            }
        } yields {
            "No possible solution"
        }
    }
}
