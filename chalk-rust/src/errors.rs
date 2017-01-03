use chalk_rust_parse::ast;

error_chain! {
    errors {
        InvalidTypeName(identifier: ast::Identifier) {
            description("invalid type name")
                display("invalid type name `{}`", identifier.str)
        }
    }
}

