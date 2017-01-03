use chalk_rust_parse::ast;

error_chain! {
    errors {
        InvalidTypeName(identifier: ast::Identifier) {
            description("invalid type name")
                display("invalid type name `{}`", identifier.str)
        }

        CannotApplyTypeParameter(identifier: ast::Identifier) {
            description("cannot apply type parameter")
                display("cannot apply type parameter `{}`", identifier.str)
        }

        IncorrectNumberOfTypeParameters(identifier: ast::Identifier,
                                        expected: usize,
                                        actual: usize) {
            description("incorrect number of type parameters")
            display("`{}` takes {} type parameters, not {}", identifier.str, expected, actual)
        }

        NotTrait(identifier: ast::Identifier) {
            description("not a trait")
            display("expected a trait, found `{}`, which is not a trait", identifier.str)
        }
    }
}

