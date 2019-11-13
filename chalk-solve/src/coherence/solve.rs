use crate::coherence::{CoherenceError, CoherenceSolver};
use crate::ext::*;
use crate::Solution;
use chalk_ir::cast::*;
use chalk_ir::family::TypeFamily;
use chalk_ir::fold::shift::Shift;
use chalk_ir::*;
use chalk_rust_ir::*;
use itertools::Itertools;

impl<TF: TypeFamily> CoherenceSolver<'_, TF> {
    pub(super) fn visit_specializations_of_trait(
        &self,
        mut record_specialization: impl FnMut(ImplId, ImplId),
    ) -> Result<(), CoherenceError> {
        // Ignore impls for marker traits as they are allowed to overlap.
        let trait_datum = self.db.trait_datum(self.trait_id);
        if trait_datum.flags.marker {
            return Ok(());
        }

        // Iterate over every pair of impls for the same trait.
        let impls = self.db.local_impls_to_coherence_check(self.trait_id);
        for (l_id, r_id) in impls.into_iter().tuple_combinations() {
            let lhs = &self.db.impl_datum(l_id);
            let rhs = &self.db.impl_datum(r_id);

            // Two negative impls never overlap.
            if !lhs.is_positive() && !rhs.is_positive() {
                continue;
            }

            // Check if the impls overlap, then if they do, check if one specializes
            // the other. Note that specialization can only run one way - if both
            // specialization checks return *either* true or false, that's an error.
            if !self.disjoint(lhs, rhs) {
                match (self.specializes(lhs, rhs), self.specializes(rhs, lhs)) {
                    (true, false) => record_specialization(l_id, r_id),
                    (false, true) => record_specialization(r_id, l_id),
                    (_, _) => {
                        let trait_name = self.db.type_name(self.trait_id.into());
                        Err(CoherenceError::OverlappingImpls(trait_name))?;
                    }
                }
            }
        }

        Ok(())
    }

    // Test if the set of types that these two impls apply to overlap. If the test succeeds, these
    // two impls are disjoint.
    //
    // We combine the binders of the two impls & treat them as existential quantifiers. Then we
    // attempt to unify the input types to the trait provided by each impl, as well as prove that
    // the where clauses from both impls all hold. At the end, we apply the `compatible` modality
    // and negate the query. Negating the query means that we are asking chalk to prove that no
    // such overlapping impl exists. By applying `compatible { G }`, chalk attempts to prove that
    // "there exists a compatible world where G is provable." When we negate compatible, it turns
    // into the statement "for all compatible worlds, G is not provable." This is exactly what we
    // want since we want to ensure that there is no overlap in *all* compatible worlds, not just
    // that there is no overlap in *some* compatible world.
    //
    // Examples:
    //
    //  Impls:
    //      impl<T> Foo for T { }
    //      impl Foo for i32 { }
    //  Generates:
    //      not { compatible { exists<T> { T = i32 } } }
    //
    //  Impls:
    //      impl<T1, U> Foo<T1> for Vec<U> { }
    //      impl<T2> Foo<T2> for Vec<i32> { }
    //  Generates:
    //      not { compatible { exists<T1, U, T2> { Vec<U> = Vec<i32>, T1 = T2 } } }
    //
    //  Impls:
    //      impl<T> Foo for Vec<T> where T: Bar { }
    //      impl<U> Foo for Vec<U> where U: Baz { }
    //  Generates:
    //      not { compatible { exists<T, U> { Vec<T> = Vec<U>, T: Bar, U: Baz } } }
    //
    fn disjoint(&self, lhs: &ImplDatum<TF>, rhs: &ImplDatum<TF>) -> bool {
        debug_heading!("overlaps(lhs={:#?}, rhs={:#?})", lhs, rhs);

        let lhs_len = lhs.binders.len();

        // Join the two impls' binders together
        let mut binders = lhs.binders.binders.clone();
        binders.extend(rhs.binders.binders.clone());

        // Upshift the rhs variables in params to account for the joined binders
        let lhs_params = params(lhs).iter().cloned();
        let rhs_params = params(rhs).iter().map(|param| param.shifted_in(lhs_len));

        // Create an equality goal for every input type the trait, attempting
        // to unify the inputs to both impls with one another
        let params_goals = lhs_params
            .zip(rhs_params)
            .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

        // Upshift the rhs variables in where clauses
        let lhs_where_clauses = lhs.binders.value.where_clauses.iter().cloned();
        let rhs_where_clauses = rhs
            .binders
            .value
            .where_clauses
            .iter()
            .map(|wc| wc.shifted_in(lhs_len));

        // Create a goal for each clause in both where clauses
        let wc_goals = lhs_where_clauses
            .chain(rhs_where_clauses)
            .map(|wc| wc.cast());

        // Join all the goals we've created together with And, then quantify them
        // over the joined binders. This is our query.
        let goal = params_goals
            .chain(wc_goals)
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("Every trait takes at least one input type")
            .quantify(QuantifierKind::Exists, binders)
            .compatible()
            .negate();

        let canonical_goal = &goal.into_closed_goal();
        let solution = self
            .solver_choice
            .into_solver()
            .solve(self.db, canonical_goal);
        let result = match solution {
            // Goal was proven with a unique solution, so no impl was found that causes these two
            // to overlap
            Some(Solution::Unique(_)) => true,
            // Goal was ambiguous, so there *may* be overlap
            Some(Solution::Ambig(_)) |
            // Goal cannot be proven, so there is some impl that causes overlap
            None => false,
        };
        debug!("overlaps: result = {:?}", result);
        result
    }

    // Test for specialization.
    //
    // If this test succeeds, the second impl specializes the first.
    //
    // Example lowering:
    //
    // more: impl<T: Clone> Foo for Vec<T>
    // less: impl<U: Clone> Foo for U
    //
    // forall<T> {
    //  if (T: Clone) {
    //    exists<U> {
    //      Vec<T> = U, U: Clone
    //    }
    //  }
    // }
    fn specializes(&self, less_special: &ImplDatum<TF>, more_special: &ImplDatum<TF>) -> bool {
        debug_heading!(
            "specializes(less_special={:#?}, more_special={:#?})",
            less_special,
            more_special
        );

        // Negative impls cannot specialize.
        if !less_special.is_positive() || !more_special.is_positive() {
            return false;
        }

        let more_len = more_special.binders.len();

        // Create parameter equality goals.
        let more_special_params = params(more_special).iter().cloned();
        let less_special_params = params(less_special).iter().map(|p| p.shifted_in(more_len));
        let params_goals = more_special_params
            .zip(less_special_params)
            .map(|(a, b)| Goal::Leaf(LeafGoal::EqGoal(EqGoal { a, b })));

        // Create the where clause goals.
        let more_special_wc = more_special
            .binders
            .value
            .where_clauses
            .iter()
            .cloned()
            .casted()
            .collect();
        let less_special_wc = less_special
            .binders
            .value
            .where_clauses
            .iter()
            .map(|wc| wc.shifted_in(more_len).cast());

        // Join all of the goals together.
        let goal = params_goals
            .chain(less_special_wc)
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("Every trait takes at least one input type")
            .quantify(QuantifierKind::Exists, less_special.binders.binders.clone())
            .implied_by(more_special_wc)
            .quantify(QuantifierKind::ForAll, more_special.binders.binders.clone());

        let canonical_goal = &goal.into_closed_goal();
        let result = match self
            .solver_choice
            .into_solver()
            .solve(self.db, canonical_goal)
        {
            Some(sol) => sol.is_unique(),
            None => false,
        };

        debug!("specializes: result = {:?}", result);

        result
    }
}

fn params<TF: TypeFamily>(impl_datum: &ImplDatum<TF>) -> &[Parameter<TF>] {
    &impl_datum.binders.value.trait_ref.parameters
}
