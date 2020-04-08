//! Tests for `WellFormed(_)` goals and clauses

use super::*;

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
fn my_test() {
    test! {
        program {
            #[lang(copy)]
            trait Copy { }

            #[lang(drop)]
            trait Drop { }

            struct S<T1, T2> where T1: Copy, T2: Copy {
                t1: T1,
                t2: T2
            }

            impl<T1, T2> Copy for S<T1, T2> { }
        }

        goal {
            compatible { not { exists<T1, T2> { S<T1, T2>: Drop } } }
        } yields {
            "No possible solution"
        }
    }
}
