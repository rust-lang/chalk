use chalk_rust_parse;
use super::LowerProgram;

fn parse_and_lower(text: &str) -> Result<ir::Program> {
    chalk_rust_parse::parse_program(text)?.lower()
}

#[test]
fn lower() {
    parse_and_lower("struct Foo { } trait Bar { } impl Bar for Foo { }").unwrap();
}
