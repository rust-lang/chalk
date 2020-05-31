#[test]
fn test_assoc_type_formatting() {
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
    reparse_test!(
    program {
        struct Foo where Foo: Baz, Foo: Bar {}
        trait Bar where Foo: Baz, dyn Baz: Bar {}
        trait Baz {}
        impl Bar for Foo where Foo: Baz, (): Baz {}
        impl Baz for Foo {}
        impl Bar for dyn Baz {}
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
  dyn [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\{\s*\}
trait [a-zA-Z0-9_-]+ \{\}
impl [a-zA-Z0-9_-]+ for [a-zA-Z0-9_-]+
where
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  \(\): [a-zA-Z0-9_-]+
\{\}
impl [a-zA-Z0-9_-]+ for [a-zA-Z0-9_-]+ \{\}
impl [a-zA-Z0-9_-]+ for dyn [a-zA-Z0-9_-]+ \{\}"#
    );
}

#[test]
fn test_assoc_ty_where_clause() {
    reparse_test!(
        program {
            trait Bar {}
            trait Fuzz {
                type Assoc
                where
                    dyn Bar: Bar,
                    Self: Bar;
            }
        }
        formatting matches
r#"trait [a-zA-Z0-9_-]+ \{\s*\}
trait [a-zA-Z0-9_-]+ \{
  type [a-zA-Z0-9_-]+
  where
    dyn [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
    [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+;
\}
"#
    );
}

#[test]
fn test_fn_where_clause() {
    reparse_test!(
        program {
            trait Bar {}
            fn foo<T>() -> T
            where
                dyn Bar: Bar,
                T: Bar;
        }
        formatting matches
r#"trait [a-zA-Z0-9_-]+ \{\s*\}
fn foo<[a-zA-Z0-9_-]+>\(\) -> [a-zA-Z0-9_-]+
where
  dyn [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+;
"#
    );
}

#[test]
fn test_name_disambiguation() {
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
