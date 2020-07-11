#[test]
fn test_assoc_type_formatting() {
    // Test associated type indentation
    // This also tests spacing around trait, struct and impl items
    reparse_test!(
        program {
            struct Foo {}
            trait Bar {
                type Assoc;
            }
            impl Bar for Foo {
                type Assoc = ();
            }
        }
        formatting matches
r#"struct [a-zA-Z0-9_-]+ \{\s*\}
trait [a-zA-Z0-9_-]+ \{
  type [a-zA-Z0-9_-]+;
\}
impl [a-zA-Z0-9_-]+ for [a-zA-Z0-9_-]+ \{
  type [a-zA-Z0-9_-]+ = \(\);
\}"#
    );
}

#[test]
fn test_struct_field_formatting() {
    // Test struct field indentation
    reparse_test!(
        program {
            struct Foo {}
            struct Bar {
                field1: Foo
            }
            struct Azg {
                field1: Foo,
                field2: Bar
            }
        }
        formatting matches
r#"struct [a-zA-Z0-9_-]+ \{\}
struct [a-zA-Z0-9_-]+ \{
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\}
struct [a-zA-Z0-9_-]+ \{
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\}"#
    );
}

#[test]
fn test_where_clause_formatting() {
    // Test where clause indentation and new-lining on impls, traits and structs
    reparse_test!(
    program {
        struct Foo where Foo: Baz, Foo: Bar {}
        trait Bar where Foo: Baz, forall<'a> dyn Baz + 'a: Bar {}
        trait Baz {}
        impl Bar for Foo where Foo: Baz, (): Baz {}
        impl Baz for Foo {}
        impl<'a> Bar for dyn Baz + 'a {}
    }
    formatting matches
r#"struct [a-zA-Z0-9_-]+
where
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\{\s*\}
trait [a-zA-Z0-9_-]+
where
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  forall<'[a-zA-Z0-9_-]+> dyn [a-zA-Z0-9_-]+ \+ '[a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\{\s*\}
trait [a-zA-Z0-9_-]+ \{\}
impl [a-zA-Z0-9_-]+ for [a-zA-Z0-9_-]+
where
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  \(\): [a-zA-Z0-9_-]+
\{\}
impl [a-zA-Z0-9_-]+ for [a-zA-Z0-9_-]+ \{\}
impl<'[a-zA-Z0-9_-]+> [a-zA-Z0-9_-]+ for dyn [a-zA-Z0-9_-]+ \+ '[a-zA-Z0-9_-]+ \{\}"#
    );
}

#[test]
fn test_assoc_ty_where_clause() {
    // Test associated ty where clause indentation (this verifies that the
    // indentation is context aware)
    reparse_test!(
        program {
            trait Bar {}
            trait Fuzz {
                type Assoc
                where
                    u32: Bar,
                    Self: Bar;
            }
        }
        formatting matches
r#"trait [a-zA-Z0-9_-]+ \{\s*\}
trait [a-zA-Z0-9_-]+ \{
  type [a-zA-Z0-9_-]+
  where
    u32: [a-zA-Z0-9_-]+,
    [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+;
\}
"#
    );
}

#[test]
fn test_fn_where_clause() {
    // Test where clause indentation, and fn type spacing
    reparse_test!(
        program {
            trait Bar {}
            fn foo<'a, T>() -> T
            where
                dyn Bar + 'a: Bar,
                T: Bar;
        }
        formatting matches
r#"trait [a-zA-Z0-9_-]+ \{\s*\}
fn foo<'[a-zA-Z0-9_-]+, [a-zA-Z0-9_-]+>\(\) -> [a-zA-Z0-9_-]+
where
  dyn [a-zA-Z0-9_-]+ \+ '[a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+;
"#
    );
}

#[test]
fn test_name_disambiguation() {
    // Tests name disambiguation, types with the same name are renamed to avoid
    // confusion. This can happen if the logging db contains types from
    // different modules.
    // we don't have modules in chalk so we can't actually test different
    // structs or traits with the same name in Chalk - but luckily our
    // implementation ignores types for name disambiguation, so we can test it
    // indirectly by using a opaque type and trait of the same name.
    reparse_test! (
        program {
            struct Foo {}
            trait Baz {}
            impl Baz for Foo {}
            opaque type Baz: Baz = Foo;
        }
        produces {
            struct Foo {}
            trait Baz {}
            impl Baz for Foo {}
            opaque type Baz_1: Baz = Foo;
        }
    );
}
