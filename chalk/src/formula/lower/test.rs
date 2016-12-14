use chalk_parse::parse_program;
use chalk_parse::ast::Span;
use super::lower_program;
use super::error::ErrorKind;

fn test(text: &str, expected: &[&str]) {
    let program = parse_program(text).unwrap();
    let clauses = lower_program("test", &program).unwrap();
    for (i, (clause, &expected_str)) in clauses.iter().zip(expected).enumerate() {
        let clause_str = format!("{:?}", clause);
        debug!("clause with index {}", i);
        debug!("actual str   = {}", clause_str);
        debug!("expected str = {}", expected_str);
        assert!(clause_str == expected_str, "expected and actual do not match");
    }
    assert_eq!(clauses.len(), expected.len());
}

fn test_err(text: &str, span: &str, kind: ErrorKind) {
    let program = parse_program(text).unwrap();
    let err = lower_program("test", &program).unwrap_err();
    let span_start = span.find(|c: char| !c.is_whitespace()).unwrap();
    let span = Span::new(span_start, span.len());
    assert!(err.span == span, "expected error span {:?}, found {:?}", span, err.span);
    assert!(err.kind == kind, "expected error kind {:?}, found {:?}", kind, err.kind);
}

#[test]
fn lower_one() {
    test("Foo(?X, _, ?Y) :- Bar(?X, _, ?Y, ?Z).",
         &[r#"forall(A, B, C, D -> implies(exists(E -> "Bar()/4"(A, E, B, C)) => "Foo()/3"(A, D, B)))"#]);
}

#[test]
fn lower_exists() {
    test("Foo(?X, _, ?Y) :- exists(?Z -> Bar(?X, _, ?Y, ?Z)).",
         &[r#"forall(A, B, C -> implies(exists(D -> exists(E -> "Bar()/4"(A, E, B, D))) => "Foo()/3"(A, C, B)))"#]);
}

#[test]
fn lower_forall() {
    test("Foo(?X, _, ?Y) :- forall(?Z -> Bar(?X, _, ?Y, ?Z)).",
         &[r#"forall(A, B, C -> implies(forall(D -> exists(E -> "Bar()/4"(A, E, B, D))) => "Foo()/3"(A, C, B)))"#]);
}

#[test]
fn lower_nested_wildcard() {
    // Test that the `_` in `Bar` could be bound to `?Z`.
    test("Foo(?X, ?Y) :- forall(?Z -> Bar(?X, ?Y, ?Z, _)).",
         &[r#"forall(A, B -> implies(forall(C -> exists(D -> "Bar()/4"(A, B, C, D))) => "Foo()/2"(A, B)))"#]);
}

#[test]
fn lower_many() {
    test("Foo(?X, _, ?Y) :- Bar(?X, _, ?Y, ?Z), Baz(?Z); Bop(?Z).",
         &[r#"forall(A, B, C, D -> implies(and(exists(E -> "Bar()/4"(A, E, B, C)), or("Baz()/1"(C); "Bop()/1"(C))) => "Foo()/3"(A, D, B)))"#]);
}

#[test]
fn lower_implies_and() {
    test("Foo(?X, _, ?Y) :- implies(Bar(?X, _, ?Y, ?Z) => Baz(?Z), Bop(?Z)).",
         &[r#"forall(A, B, C, D -> implies(implies(forall(E -> "Bar()/4"(A, E, B, C)) => and("Baz()/1"(C), "Bop()/1"(C))) => "Foo()/3"(A, D, B)))"#]);
}

#[test]
fn lower_implies_or() {
    test("Foo(?X, _, ?Y) :- implies(Bar(?X, _, ?Y, ?Z) => Baz(?Z); Bop(?Z)).",
         &[r#"forall(A, B, C, D -> implies(implies(forall(E -> "Bar()/4"(A, E, B, C)) => or("Baz()/1"(C); "Bop()/1"(C))) => "Foo()/3"(A, D, B)))"#]);
}

#[test]
fn lower_implies_or_in_clause() {
    test_err("Foo(?X, _, ?Y) :- implies(Bar(?X, _, ?Y, ?Z); Bop(?Z) => Baz(?Z)).",
             "                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^",
             ErrorKind::OrInClause);
}

#[test]
fn lower_and_clause() {
    test("Foo(?X, _, ?Y) :- implies(Foo(?X), Bar(?Y) => Baz(?X, ?Y)).",
         &[r#"forall(A, B, C -> implies(implies("Foo()/1"(A), "Bar()/1"(B) => "Baz()/2"(A, B)) => "Foo()/3"(A, C, B)))"#]);
}
