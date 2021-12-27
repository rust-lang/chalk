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
            expect![["No possible solution"]]
        }
        goal {
            forall<T> {
                if (T: Send) {
                    List<T>: Send
                }
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            List<TypeA>: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            exists<T> {
                T: Send
            }
        } yields {
            expect![["Ambiguous; no inference guidance"]]
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
            expect![["Unique"]]
        }

        // No fields so `Useless<T>` is `Send`.
        goal {
            forall<T> {
                Useless<T>: Send
            }
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                if (T: Send) {
                    Data<T>: Send
                }
            }
        } yields {
            expect![["Unique"]]
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
            expect![["No possible solution"]]
        }

        goal {
            TypeB: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            Vec<TypeA>: Send
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            Vec<TypeB>: Send
        } yields {
            expect![["Unique"]]
        }

        goal {
            forall<T> {
                Vec<T>: Send
            }
        } yields {
            expect![["No possible solution"]]
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
            expect![["Floundered"]]
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
            expect![["Unique"]]
        }

        goal {
            B: Send
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            C: Send
        } yields {
            expect![["No possible solution"]]
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

        // The following types only contain AutoTrait-types, and thus implement AutoTrait themselves.
        goal { (i32, f32): AutoTrait }
        yields { expect![["Unique"]] }

        goal { [(); 1]: AutoTrait }
        yields { expect![["Unique"]] }

        goal { [()]: AutoTrait }
        yields { expect![["Unique"]] }

        goal { u32: AutoTrait }
        yields { expect![["Unique"]] }

        goal { *const (): AutoTrait }
        yields { expect![["Unique"]] }

        goal { *mut (): AutoTrait }
        yields { expect![["Unique"]] }

        goal { forall<'a> { &'a (): AutoTrait } }
        yields { expect![["Unique"]] }

        goal { forall<'a> { &'a mut (): AutoTrait } }
        yields { expect![["Unique"]] }

        goal { str: AutoTrait }
        yields { expect![["Unique"]] }

        goal { !: AutoTrait }
        yields { expect![["Unique"]] }

        goal { Enum: AutoTrait }
        yields { expect![["Unique"]] }

        goal { func: AutoTrait }
        yields { expect![["Unique"]] }

        goal { good_closure: AutoTrait }
        yields { expect![["Unique"]] }

        goal { fn(Marker) -> Marker: AutoTrait }
        yields { expect![["Unique"]] }


        // foreign types do not implement AutoTraits automatically
        goal { Ext: AutoTrait }
        yields { expect![["No possible solution"]] }

        // The following types do contain non-AutoTrait types, and thus do not implement AutoTrait.
        goal { bad_closure: AutoTrait }
        yields { expect![["No possible solution"]] }

        goal { ExtEnum: AutoTrait }
        yields { expect![["No possible solution"]] }

        goal { (Struct, Marker): AutoTrait }
        yields { expect![["No possible solution"]] }
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
            expect![["Unique"]]
        }

        goal {
            No: AutoTrait
        }
        yields {
            expect![["No possible solution"]]
        }

        goal {
            X: AutoTrait
        }
        yields {
            expect![["Unique"]]
        }

        goal {
            WrapperNo<Yes>: AutoTrait
        }
        yields {
            expect![["No possible solution"]]
        }

        goal {
            WrapperYes<No>: AutoTrait
        }
        yields {
            expect![["No possible solution"]]
        }
    }
}

#[test]
fn phantom_auto_trait() {
    test! {
        program {
            #[auto] trait AutoTrait {}
            #[phantom_data] struct PhantomData<T> {}
            struct Bad {}
            impl !AutoTrait for Bad {}
        }

        goal {
            PhantomData<Bad>: AutoTrait
        }
        yields {
            expect![["No possible solution"]]
        }
    }
}
