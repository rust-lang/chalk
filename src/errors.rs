use chalk_parse::{self, ast};
use chalk_ir;
use rust_ir;

error_chain! {
    links {
        Parse(chalk_parse::errors::Error, chalk_parse::errors::ErrorKind);
    }

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

        OverlappingImpls(trait_id: chalk_ir::Identifier) {
            description("overlapping impls")
            display("overlapping impls of trait {:?}", trait_id)
        }

        IllFormedTypeDecl(ty_id: chalk_ir::Identifier) {
            description("ill-formed type declaration")
            display("type declaration {:?} does not meet well-formedness requirements", ty_id)
        }

        IllFormedTraitImpl(trait_id: chalk_ir::Identifier) {
            description("ill-formed trait impl")
            display("trait impl for {:?} does not meet well-formedness requirements", trait_id)
        }

        CouldNotMatch {
            description("could not match")
                display("could not match")
        }

        DuplicateLangItem(item: rust_ir::LangItem) {
            description("Duplicate lang item")
                display("Duplicate lang item `{:?}`", item)
        }

        FailedOrphanCheck(trait_id: chalk_ir::Identifier) {
            description("impl violates the orphan rules")
                display("impl for trait {:?} violates the orphan rules", trait_id)
        }
    }
}
