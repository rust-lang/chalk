use super::*;

#[test]
fn opaque_bounds() {
    test! {
        program {
            struct Ty { }

            trait Clone { }

            impl Clone for Ty { }

            opaque type T: Clone = Ty;
        }

        goal {
            T: Clone
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn opaque_reveal() {
    test! {
        program {
            struct Ty { }
            trait Trait { }
            impl Trait for Ty { }

            trait Clone { }
            impl Clone for Ty { }
            opaque type T: Clone = Ty;
        }

        goal {
            if (Reveal) {
                T: Trait
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            T: Trait
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn opaque_where_clause() {
    test! {
        program {
            struct Ty { }

            trait Clone { }
            impl Clone for Ty { }

            trait Trait { }
            impl Trait for Ty { }

            opaque type T: Clone where T: Trait = Ty;

            struct Vec<U> { }

            impl<V> Clone for Vec<V> { }
            impl<U> Trait for Vec<U> { }

            opaque type S<U>: Clone where U: Trait = Vec<U>;
        }

        goal {
            if (T: Trait) {
                WellFormed(T)
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed(T)
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            forall<U> {
                if (U : Trait) {
                    WellFormed(S<U>)
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<U> {
                WellFormed(S<U>)
            }
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn opaque_generics_simple() {
    test! {
        program {
            trait Iterator { type Item; }

            struct Vec<T> { }
            struct Bar { }
            impl<T> Iterator for Vec<T> {
                type Item = u32;
            }

            opaque type Foo<X>: Iterator = Vec<X>;
        }

        goal {
            Foo<Bar>: Iterator
        } yields {
            expect![["Unique"]]
        }

    }
}

#[test]
fn opaque_generics() {
    test! {
        program {
            trait Iterator { type Item; }

            struct Vec<T> { }
            struct Bar { }

            impl<T> Iterator for Vec<T> {
                type Item = T;
            }

            opaque type Foo<X>: Iterator<Item = X> = Vec<X>;
        }

        goal {
            Foo<Bar>: Iterator<Item = Bar>
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                Foo<T>: Iterator<Item = T>
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> {
                <Foo<Bar> as Iterator>::Item = T
            }
        } yields[SolverChoice::slg_default()] {
            expect![["Ambiguous; no inference guidance"]] // #234
        } yields[SolverChoice::recursive_default()] {
            expect![["Unique; substitution [?0 := Bar]"]]
        }
    }
}

#[test]
fn opaque_trait_generic() {
    test! {
        program {
            trait Trait<T> {}
            struct Foo {}
            impl Trait<u32> for Foo {}

            opaque type Bar: Trait<u32> = Foo;
        }

        goal {
            exists<T> {
                Bar: Trait<T>
            }
        } yields {
            expect![["Unique; substitution [?0 := Uint(U32)]"]]
        }
    }
}

#[test]
fn opaque_auto_traits() {
    test! {
        program {
            struct Bar { }
            struct Baz { }
            trait Trait { }

            impl Trait for Bar { }
            impl Trait for Baz { }

            #[auto]
            trait Send { }

            impl !Send for Baz { }

            opaque type Opaque1: Trait = Bar;
            opaque type Opaque2: Trait = Baz;
        }

        goal {
            Opaque1: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            Opaque2: Send
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn opaque_auto_traits_indirect() {
    test! {
        program {
            struct Bar { }
            struct Baz { }
            trait Trait { }

            impl Trait for Bar { }
            impl Trait for Baz { }

            #[auto]
            trait Send { }
            trait SendDerived where Self: Send { }

            impl<T> SendDerived for T where T: Send { }

            impl !Send for Baz { }

            opaque type Opaque1: Trait = Bar;
            opaque type Opaque2: Trait = Baz;
        }

        goal {
            Opaque1: SendDerived
        } yields {
            expect![["Unique"]]
        }

        goal {
            Opaque2: SendDerived
        } yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn opaque_super_trait() {
    test! {
        program {
            trait Base {}
            trait Super where Self: Base {}
            impl Base for () {}
            impl Super for () {}

            opaque type Opaque: Super = ();
        }

        goal {
            Opaque: Base
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn opaque_assoc_in_super_trait_bounds() {
    test! {
        program {
            trait Foo {
                type A;
            }
            trait EmptyFoo where Self: Foo<A = ()> { }
            impl Foo for i32 {
                type A = ();
            }
            impl<T> EmptyFoo for T where T: Foo<A = ()> { }

            opaque type T: EmptyFoo = i32;
        }

        goal {
            T: EmptyFoo
        } yields {
            expect![[r#"Unique"#]]
        }

        goal {
            T: Foo
        } yields {
            expect![[r#"Unique"#]]
        }
    }
}
