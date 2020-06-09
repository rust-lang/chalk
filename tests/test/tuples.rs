use super::*;

#[test]
fn tuple_trait_impl() {
    test! {
        program {
            trait Foo { }
            struct S1 { }
            impl Foo for (S1, S1) { }
            impl Foo for () { }
        }
        goal {
            (S1, S1): Foo
        } yields {
            "Unique"
        }

        goal {
            (): Foo
        } yields {
            "Unique"
        }
    }
    test! {
        program {
            trait Foo { }
            impl Foo for (i32, i32, (i32,)) { }
        }

        goal {
            (i32, i32, (i32, )): Foo
        } yields {
            "Unique"
        }
    }
}
