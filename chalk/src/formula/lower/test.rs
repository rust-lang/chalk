use chalk_parse::parse_program;
use chalk_parse::ast::Span;
use super::lower_program;
use super::ErrorKind;

fn test(text: &str, expected: &[&str]) {
    let program = parse_program(text).unwrap();
    let clauses = lower_program(&program).unwrap();
    for (i, (clause, &expected_str)) in clauses.iter().zip(expected).enumerate() {
        let clause_str = format!("{:?}", clause);
        println!("clause with index {}", i);
        println!("actual str   = {}", clause_str);
        println!("expected str = {}", expected_str);
        assert!(clause_str == expected_str, "expected and actual do not match");
    }
    assert_eq!(clauses.len(), expected.len());
}

fn test_err(text: &str, span: &str, kind: ErrorKind) {
    let program = parse_program(text).unwrap();
    let err = lower_program(&program).unwrap_err();
    let span_start = span.find(|c: char| !c.is_whitespace()).unwrap();
    let span = Span::new(span_start, span.len());
    assert!(err.span == span, "expected error span {:?}, found {:?}", span, err.span);
    assert!(err.kind == kind, "expected error kind {:?}, found {:?}", kind, err.kind);
}

#[test]
fn lower_one() {
    test("Foo(X, _, Y) :- Bar(X, _, Y, Z).",
         &[r#"forall(A, B, C -> implies(exists(D -> "Bar()/4"(A, D, B, C)) => forall(D -> "Foo()/3"(A, D, B))))"#]);
}

#[test]
fn lower_exists() {
    test("Foo(X, _, Y) :- exists(Z -> Bar(X, _, Y, Z)).",
         &[r#"forall(A, B -> implies(exists(C -> exists(D -> "Bar()/4"(A, D, B, C))) => forall(C -> "Foo()/3"(A, C, B))))"#]);
}

#[test]
fn lower_forall() {
    test("Foo(X, _, Y) :- forall(Z -> Bar(X, _, Y, Z)).",
         &[r#"forall(A, B -> implies(forall(C -> exists(D -> "Bar()/4"(A, D, B, C))) => forall(C -> "Foo()/3"(A, C, B))))"#]);
}

#[test]
fn lower_many() {
    test("Foo(X, _, Y) :- Bar(X, _, Y, Z), Baz(Z); Bop(Z).",
         &[r#"forall(A, B, C -> implies(and(exists(D -> "Bar()/4"(A, D, B, C)), or("Baz()/1"(C); "Bop()/1"(C))) => forall(D -> "Foo()/3"(A, D, B))))"#]);
}

#[test]
fn lower_implies_and() {
    test("Foo(X, _, Y) :- implies(Bar(X, _, Y, Z) => Baz(Z), Bop(Z)).",
         &[r#"forall(A, B, C -> implies(implies(forall(D -> "Bar()/4"(A, D, B, C)) => and("Baz()/1"(C), "Bop()/1"(C))) => forall(D -> "Foo()/3"(A, D, B))))"#]);
}

#[test]
fn lower_implies_or() {
    test("Foo(X, _, Y) :- implies(Bar(X, _, Y, Z) => Baz(Z); Bop(Z)).",
         &[r#"forall(A, B, C -> implies(implies(forall(D -> "Bar()/4"(A, D, B, C)) => or("Baz()/1"(C); "Bop()/1"(C))) => forall(D -> "Foo()/3"(A, D, B))))"#]);
}

#[test]
fn lower_implies_or_in_clause() {
    test_err("Foo(X, _, Y) :- implies(Bar(X, _, Y, Z); Bop(Z) => Baz(Z)).",
             "                        ^^^^^^^^^^^^^^^^^^^^^^^",
             ErrorKind::OrInClause);
}
