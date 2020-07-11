use super::*;

#[test]
fn test_simple_trait() {
    // Simplest test for 'trait'
    reparse_test!(
        program {
            trait Foo {}
        }
    );
}

#[test]
fn test_generic_trait() {
    // Test we can print generics introduced by a trait
    reparse_test!(
        program {
            trait Foo<T> {}
            trait Bar<T, U> {}
        }
    );
}

#[test]
fn test_trait_where_clauses() {
    // Test printing trait where clauses
    reparse_test!(
        program {
            trait Foo<T> where T: Trait {}
            trait Trait {}
        }
    );
}

#[test]
fn test_basic_trait_impl() {
    // Test simplest trait implementation
    reparse_test!(
        program {
            struct Foo { }
            trait Bar {}
            impl Bar for Foo { }
        }
    );
}

#[test]
fn test_trait_flags() {
    // Test every individual flag that can appear on a trait, as well as the
    // combination of all of them. We test the combination to ensure that we
    // satisfy any ordering requirements present.
    let flags = vec![
        "auto",
        "marker",
        "upstream",
        "fundamental",
        "non_enumerable",
        "coinductive",
        "object_safe",
    ];
    reparse_test(&format!(
        "{}trait Hello {{}}",
        flags
            .iter()
            .map(|f| format!("#[{}]", f))
            .collect::<Vec<_>>()
            .join("\n")
    ));
    for flag in flags {
        reparse_test(&format!(
            "
            #[{0}]
            trait Hello_{0} {{}}
            ",
            flag
        ));
    }
}

#[test]
fn test_wellknown_traits() {
    // Test all possible `#[lang]` attributes on traits.
    let well_knowns = vec![
        "sized", "copy", "clone", "drop", "fn_once", "fn_mut", "fn", "unsize",
    ];
    for flag in well_knowns {
        reparse_test(&format!(
            "
            #[lang({0})]
            trait Hello_{0} {{}}
            ",
            flag
        ));
    }
}

#[test]
fn test_lang_with_flag() {
    // Test we output the correct ordering when printing a trait with both flags
    // and a #[lang] attribute.
    reparse_test!(
        program {
            #[auto]
            #[lang(sized)]
            trait Foo {

            }
        }
    );
}
