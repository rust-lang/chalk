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
            expect![["No possible solution"]]
        }

        goal {
            WellFormed(Foo<Baz>)
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed(Foo<Foo<Baz>>)
        } yields {
            expect![["Unique"]]
        }
    }
}

#[test]
fn enum_wf() {
    test! {
        program {
            enum Foo<T> where T: Eq { }
            enum Bar { }
            enum Baz { }

            trait Eq { }

            impl Eq for Baz { }
            impl<T> Eq for Foo<T> where T: Eq { }
        }

        goal {
            WellFormed(Foo<Bar>)
        } yields {
            expect![["No possible solution"]]
        }

        goal {
            WellFormed(Foo<Baz>)
        } yields {
            expect![["Unique"]]
        }

        goal {
            WellFormed(Foo<Foo<Baz>>)
        } yields {
            expect![["Unique"]]
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
            expect![["No possible solution"]]
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
            expect![["Unique"]]
        }
    }
}

#[test]
fn placeholder_wf() {
    test! {
        goal {
            forall<T> { WellFormed(T) }
        } yields {
            expect![["Unique"]]
        }
    }
}
