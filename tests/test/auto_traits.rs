//! Tests targeting auto traits specifically

use super::*;

#[test]
fn auto_semantics() {
    test! {
        program {
            trait Sized { }
            #[auto] trait Send { }

            struct TypeA { }

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
            List<TypeA>: Send
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
fn auto_trait_without_impls() {
    test! {
        program {
            #[auto] trait Send { }

            struct TypeA { }

            struct Useless<T> { }

            struct Data<T> {
                data: T
            }
        }

        goal {
            TypeA: Send
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

            struct TypeA { }
            struct TypeB { }
            struct Vec<T> { }

            impl<T> Send for Vec<T> where T: Send { }
            impl !Send for TypeA { }
        }

        goal {
            TypeA: Send
        } yields {
            "No possible solution"
        }

        goal {
            TypeB: Send
        } yields {
            "Unique"
        }

        goal {
            Vec<TypeA>: Send
        } yields {
            "No possible solution"
        }

        goal {
            Vec<TypeB>: Send
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

/// This Flounders because auto traits can't be enumerated
#[test]
fn auto_traits_flounder() {
    test! {
        program {
            struct Foo { }
            struct Bar { }

            #[auto]
            trait Send { }
        }

        goal {
            exists<A> { A: Send }
        } yields_first[SolverChoice::slg(3, None)] {
            "Floundered"
        }
    }
}

#[test]
fn enum_auto_trait() {
    test! {
        program {
            #[auto] trait Send { }
            struct Foo { }
            struct Bar { }
            impl Send for Foo { }
            impl !Send for Bar { }

            enum A {
                X,
                Y(Foo),
                Z {
                    z: Foo,
                }
            }

            enum B {
                X,
                Y(Foo),
                Z {
                    z: Bar,
                }
            }

            enum C {
                X,
                Y(Bar),
                Z {
                    z: Foo,
                }
            }
        }

        goal {
            A: Send
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            B: Send
        } yields {
            "No possible solution"
        }

        goal {
            C: Send
        } yields {
            "No possible solution"
        }
    }
}

#[test]
fn builtin_auto_trait() {
    test! {
        program {
            #[auto] trait AutoTrait {}
            struct Struct {}
            enum Enum { Var1, Var2 }
            fn func();

            struct Marker {}
            impl !AutoTrait for Marker {}

            closure good_closure(self, arg: Marker) -> Marker { i32 }
            closure bad_closure(self, arg: i32) -> i32  { Marker }

            extern type Ext;
            enum ExtEnum { GoodVariant, BadVariant(Ext) }
        }

        goal {
            (Struct, Marker): AutoTrait
        }
        yields {
            "No possible solution"
        }

        goal {
            forall<'a> { (fn(), [(); 1], [()], u32, *const (), str, !, Struct, Enum, func, good_closure, &'a ()): AutoTrait }
        }
        yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            good_closure: AutoTrait
        }
        yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            bad_closure: AutoTrait
        }
        yields {
            "No possible solution"
        }

        goal {
            ExtEnum: AutoTrait
        }
        yields {
            "No possible solution"
        }
    }
}

#[test]
fn adt_auto_trait() {
    test! {
        program {
            #[auto] trait AutoTrait {}
            struct Yes {}
            struct No {}
            impl !AutoTrait for No {}

            struct WrapperNo<T> { t: T }
            struct WrapperYes<T> { t: T }

            struct X {}
            impl !AutoTrait for WrapperNo<X> {}
        }

        goal {
            Yes: AutoTrait
        }
        yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            No: AutoTrait
        }
        yields {
            "No possible solution"
        }

        goal {
            X: AutoTrait
        }
        yields {
            "Unique; substitution [], lifetime constraints []"
        }

        goal {
            WrapperNo<Yes>: AutoTrait
        }
        yields {
            "No possible solution"
        }

        goal {
            WrapperYes<No>: AutoTrait
        }
        yields {
            "No possible solution"
        }
    }
}
