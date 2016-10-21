use chalk_parse::parse_program;
use super::lower_program;

fn test(text: &str) {
    let program = parse_program(text).unwrap();
    let clauses = lower_program(&program).unwrap();
    println!("{:#?}", clauses);
    assert!(false);
}

#[test]
fn lower_one() {
    test("Foo(X, _, Y) :- Bar(X, _, Y, Z).");
}
