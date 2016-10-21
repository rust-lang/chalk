use chalk_parse::parse_program;
use super::lower_program;

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

#[test]
fn lower_one() {
    test("Foo(X, _, Y) :- Bar(X, _, Y, Z).",
         &[r#"forall(A, B, C -> (exists(D -> "Bar()/4"(A, D, B, C)) => forall(D -> "Foo()/3"(A, D, B))))"#]);
}
