use super::*;

#[test]
fn test_assoc_type_formatting() {
    test_formatting(
        "
        struct Foo {}
        trait Bar {
            type Assoc;
        }
        impl Bar for Foo {
            type Assoc = ();
        }
        ",
        r#"struct [a-zA-Z0-9_-]+ \{\s*\}
trait [a-zA-Z0-9_-]+ \{
  type [a-zA-Z0-9_-]+;
\}
impl [a-zA-Z0-9_-]+ for [a-zA-Z0-9_-]+ \{
  type [a-zA-Z0-9_-]+ = \(\);
\}"#,
    );
}

#[test]
fn test_struct_field_formatting() {
    test_formatting(
        "
        struct Foo {}
        struct Bar {
            field1: Foo
        }
        struct Azg {
            field1: Foo,
            field2: Bar
        }
        ",
        r#"struct [a-zA-Z0-9_-]+ \{\}
struct [a-zA-Z0-9_-]+ \{
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\}
struct [a-zA-Z0-9_-]+ \{
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+,
  [a-zA-Z0-9_-]+: [a-zA-Z0-9_-]+
\}"#,
    );
}

#[test]
fn test_where_clause_formatting() {
    test_formatting(
        "
        struct Foo where Foo: Baz, Foo: Bar {}
        trait Bar where Foo: Baz, dyn Baz: Bar {}
        trait Baz {}
        impl Bar for Foo where Foo: Baz, (): Baz {}
        impl Baz for Foo {}
        impl Bar for dyn Baz {}
    ",
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
impl [a-zA-Z0-9_-]+ for dyn [a-zA-Z0-9_-]+ \{\}"#,
    );
}
