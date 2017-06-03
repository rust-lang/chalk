use std::sync::Arc;

use itertools::Itertools;

use errors::*;
use ir::*;
use solve::solver::Solver;

impl Program {
    pub fn check_overlapping_impls(&self) -> Result<()> {
        let mut solver = Solver::new(&Arc::new(self.environment()), 10);

        // Group impls by trait.
        let impl_groupings = self.impl_data.iter().group_by(|&(_, impl_datum)| {
            impl_datum.binders.value.trait_ref.trait_id
        });

        for (trait_id, impls) in &impl_groupings {
            // Get all the pairs of impls from this trait
            let impls: Vec<&ImplDatum> = impls.map(|(_, impl_datum)| impl_datum).collect();

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
fn intersection_of(lhs: &ImplDatum, rhs: &ImplDatum) -> Canonical<InEnvironment<Goal>> {
    fn params(impl_datum: &ImplDatum) -> &[Parameter] {
        &impl_datum.binders.value.trait_ref.parameters
    }

    debug_assert!(params(lhs).len() == params(rhs).len());

    // Join the two impls' binders together 
    let mut binders = lhs.binders.binders.clone();
    binders.extend(rhs.binders.binders.clone());

    // Upshift the rhs variables to account for the joined binders
    let lhs_params = params(lhs).iter().cloned();
    let rhs_params = params(rhs).iter().map(|param| param.up_shift(lhs.binders.len()));

    // Create an equality goal of inputs to the trait, attempting to unify
    // the inputs to both impls with each other
    let goal = lhs_params.zip(rhs_params)
                .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })))
                .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                .expect("Every trait takes at least one input type");

    Canonical {
        value: InEnvironment {
            environment: Environment::new(),
            goal: Goal::Quantified(QuantifierKind::Exists, Binders {
                value: Box::new(goal),
                binders: binders,
            }),
        },
        binders: vec![],
    }
}
