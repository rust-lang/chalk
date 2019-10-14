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
