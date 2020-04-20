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
fn drop_compatible() {
    test! {
        program {
            #[lang(drop)]
            trait Drop { }

            struct S<T> { }
        }

        goal {
            compatible { not { exists<T> { S<T>: Drop } } }
        } yields {
            "Unique; substitution [], lifetime constraints []"
        }
    }
}
