use cast::Cast;
use fold::Subst;
use ir::*;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Environment {
    pub universe: UniverseIndex,
    pub clauses: Vec<WhereClause>,
}

impl Environment {
    pub fn new() -> Arc<Environment> {
        Arc::new(Environment { universe: UniverseIndex::root(), clauses: vec![] })
    }

    pub fn add_clauses<I>(&self, clauses: I) -> Arc<Environment>
        where I: IntoIterator<Item = WhereClause>
    {
        let mut env = self.clone();
        env.clauses.extend(clauses);
        Arc::new(env)
    }

    pub fn new_universe(&self) -> Arc<Environment> {
        let mut env = self.clone();
        env.universe = UniverseIndex { counter: self.universe.counter + 1 };
        Arc::new(env)
    }

    pub fn elaborated_clauses(&self, program: &Program) -> impl Iterator<Item = WhereClause> {
        let mut set = HashSet::new();
        set.extend(self.clauses.iter().cloned());

        let mut stack: Vec<_> = set.iter().cloned().collect();

        while let Some(clause) = stack.pop() {
            let mut push_clause = |clause: WhereClause| {
                if !set.contains(&clause) {
                    set.insert(clause.clone());
                    stack.push(clause);
                }
            };

            match clause {
                WhereClause::Implemented(ref trait_ref) => {
                    // trait Foo<A> where Self: Bar<A> { }
                    // T: Foo<U>
                    // ----------------------------------------------------------
                    // T: Bar<U>

                    let trait_data = &program.trait_data[&trait_ref.trait_id];
                    for where_clause in &trait_data.where_clauses {
                        let where_clause = Subst::apply(&trait_ref.parameters, where_clause);
                        push_clause(where_clause);
                    }
                }
                WhereClause::Normalize(Normalize { ref projection, ty: _ }) => {
                    // <T as Trait<U>>::Foo ===> V
                    // ----------------------------------------------------------
                    // T: Trait<U>

                    let (associated_ty_data, trait_params, _) = program.split_projection(projection);
                    let trait_ref = TraitRef {
                        trait_id: associated_ty_data.trait_id,
                        parameters: trait_params.to_owned()
                    };
                    push_clause(trait_ref.cast());
                }
            }
        }

        set.into_iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InEnvironment<G> {
    pub environment: Arc<Environment>,
    pub goal: G,
}

impl<G> InEnvironment<G> {
    pub fn new(environment: &Arc<Environment>, goal: G) -> Self {
        InEnvironment { environment: environment.clone(), goal }
    }

    pub fn map<OP, H>(self, op: OP) -> InEnvironment<H>
        where OP: FnOnce(G) -> H
    {
        InEnvironment {
            environment: self.environment,
            goal: op(self.goal),
        }
    }
}
