use crate::WhereClause;
use chalk_ir::cast::Cast;
use chalk_ir::family::ChalkIr;
use chalk_ir::{DomainGoal, ProjectionEq, TraitRef};

impl Cast<WhereClause> for TraitRef<ChalkIr> {
    fn cast(self) -> WhereClause {
        WhereClause::Implemented(self)
    }
}

impl Cast<WhereClause> for ProjectionEq<ChalkIr> {
    fn cast(self) -> WhereClause {
        WhereClause::ProjectionEq(self)
    }
}

impl Cast<DomainGoal<ChalkIr>> for WhereClause {
    fn cast(self) -> DomainGoal<ChalkIr> {
        match self {
            WhereClause::Implemented(t) => DomainGoal::Implemented(t),
            WhereClause::ProjectionEq(t) => DomainGoal::ProjectionEq(t),
        }
    }
}
