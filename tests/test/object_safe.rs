use super::*;

#[test]
fn object_safe_flag() {
    test! {
        program {
            #[object_safe]
            trait Foo {}
            trait Bar {}
        }

        goal { ObjectSafe(Foo) } yields { "Unique" }
        goal { not { ObjectSafe(Bar) } } yields { "Unique" }
    }
}
