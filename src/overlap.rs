use std::sync::Arc;

use itertools::Itertools;

use errors::*;
use ir::*;
use solve::solver::Solver;

impl Program {
    pub fn check_overlapping_impls(&self) -> Result<()> {
        let mut solver = Solver::new(&Arc::new(self.environment()), 10);

        // Create a vector of references to impl datums, sorted by trait ref
        let impl_data = self.impl_data.values().sorted_by(|lhs, rhs| {
            lhs.binders.value.trait_ref.trait_id.cmp(&rhs.binders.value.trait_ref.trait_id)
        });

        // Group impls by trait.
        let impl_groupings = impl_data.into_iter().group_by(|impl_datum| {
            impl_datum.binders.value.trait_ref.trait_id
        });

        for (trait_id, impls) in &impl_groupings {
            let impls: Vec<&ImplDatum> = impls.collect();

            // For each pair, check their overlap by generating an "intersection"
            // goal. In this case, success is an error - it means that there is at
            // least one type in the intersection of these two impls.
            for (lhs, rhs) in impls.into_iter().tuple_combinations() {
                match solver.solve_goal(intersection_of(lhs, rhs)) {
                    Ok(_)   => {
                        let trait_id = self.type_kinds.get(&trait_id).unwrap().name;
                        Err(Error::from_kind(ErrorKind::OverlappingImpls(trait_id)))
                    }
                    Err(_)  => Ok(())
                }?;
            }
        }

        Ok(())
    }
}

// The goal to test overlap.
//
// If this goal succeeds, these two impls overlap.
//
// We combine the binders of the two impls & treat them as existential
// quantifiers. Then we attempt to unify the input types to treat provided
// by each impl, as well as prove that the where clauses from both impls all
// hold.
//
// Examples:
//
//  Impls:
//      impl<T> Foo for T { }
//      impl Foo for i32 { }
//  Generates:
//      exists<T> { T = i32 }
//
//  Impls:
//      impl<T1, U> Foo<T1> for Vec<U> { }
//      impl<T2> Foo<T2> for Vec<i32> { }
//  Generates:
//      exists<T1, U, T2> { U = i32, T1 = T2 }
//
//
//  Impls:
//      impl<T> Foo for Vec<T> where T: Bar { }
//      impl<U> Foo for Vec<U> where U: Baz { }
//  Generates:
//      exists<T, U> { T = U, T: Bar, U: Baz }
//
fn intersection_of(lhs: &ImplDatum, rhs: &ImplDatum) -> Canonical<InEnvironment<Goal>> {
    fn params(impl_datum: &ImplDatum) -> &[Parameter] {
        &impl_datum.binders.value.trait_ref.parameters
    }

    debug_assert!(params(lhs).len() == params(rhs).len());

    let lhs_len = lhs.binders.len();

    // Join the two impls' binders together 
    let mut binders = lhs.binders.binders.clone();
    binders.extend(rhs.binders.binders.clone());

    // Upshift the rhs variables in params to account for the joined binders
    let lhs_params = params(lhs).iter().cloned();
    let rhs_params = params(rhs).iter().map(|param| param.up_shift(lhs_len));

    // Create an equality goal for every input type the trait, attempting
    // to unify the inputs to both impls with one another
    let params_goals = lhs_params.zip(rhs_params)
                        .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

    // Upshift the rhs variables in where clauses
    let lhs_where_clauses = lhs.binders.value.where_clauses.iter().cloned();
    let rhs_where_clauses = rhs.binders.value.where_clauses.iter().map(|wc| wc.up_shift(lhs_len));

    // Create a goal for each clause in both where clauses
    let wc_goals = lhs_where_clauses.chain(rhs_where_clauses)
                .map(|wc| Goal::Leaf(LeafGoal::DomainGoal(wc)));
 
    // Join all the goals we've created together with And, then quantify them
    // over the joined binders. This is our query.
    let goal = params_goals.chain(wc_goals)
                .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                .expect("Every trait takes at least one input type")
                .quantify(QuantifierKind::Exists, binders);

    Canonical {
        value: InEnvironment::empty(goal),
        binders: vec![],
    }
}
