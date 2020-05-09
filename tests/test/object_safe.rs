use super::*;

/// The current implementation of RustIrDatabase::is_object_safe only checks for
/// the #[object_safe] flag. This test will start failing once it performs a
/// proper test because Foo should be object safe.
#[test]
fn by_default_not_object_safe() {
    test! {
        program {
            trait Foo {}
        }

        goal { ObjectSafe(Foo) } yields { "No possible solution" }
        goal { not { ObjectSafe(Foo) } } yields { "Unique" }
    }
}

#[test]
fn object_safe_flag() {
    test! {
        program {
            #[object_safe]
            trait Foo {}
        }

        goal { ObjectSafe(Foo) } yields { "Unique" }
        goal { not { ObjectSafe(Foo) } } yields { "No possible solution" }
    }
}
