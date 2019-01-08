use chalk_ir::*;
use chalk_ir::cast::{Cast, Caster};

crate fn program_clauses_for_dynamic_types(goal: &DomainGoal) -> impl Iterator<Item = ProgramClause> {
    let mut clauses = vec![];

    match goal {
        DomainGoal::WellFormed(WellFormed::Ty(Ty::Dynamic(traits))) => {
            let ty = Ty::Dynamic(traits.clone());

            // ```
            // WellFormed(dyn (Foo + Bar)) :-
            //    Shallow(dyn (Foo + Bar): Foo),
            //    Shallow(dyn (Foo + Bar): Bar),
            //    ObjectSafe(Foo),
            //    ObjectSafe(Bar).
            // ```
            let mut conditions: Vec<_> = traits
                .iter()
                .cloned()
                .map(|tr| DomainGoal::Shallow(tr.as_trait_ref(ty.clone())))
                .casted()
                .collect();
            
            conditions.extend(
                traits.iter().map(|tr| DomainGoal::ObjectSafe(tr.trait_id)).casted()
            );

            let program_clause = ProgramClauseImplication {
                consequence: goal.clone(),
                conditions,
            }.cast();

            clauses.push(program_clause);
        }

        DomainGoal::Holds(WhereClause::Implemented(trait_ref)) => {
            if let Ty::Dynamic(_traits) = trait_ref.parameters[0].assert_ty_ref() {
                // FIXME: add generated impls for super-traits bounds derived from `traits`.
            }
        }

        _ => (),
    }

    clauses.into_iter()
}
