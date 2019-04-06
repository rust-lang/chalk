use std::sync::Arc;

use crate::program_environment::ProgramEnvironment;
use crate::rules::RustIrSource;
use chalk_ir::cast::*;
use chalk_ir::fold::shift::Shift;
use chalk_ir::fold::*;
use chalk_ir::*;
use chalk_rust_ir::*;
use chalk_solve::ext::*;
use chalk_solve::solve::SolverChoice;
use failure::Fail;
use itertools::Itertools;

mod test;

#[derive(Fail, Debug)]
pub enum WfError {
    #[fail(
        display = "type declaration {:?} does not meet well-formedness requirements",
        _0
    )]
    IllFormedTypeDecl(chalk_ir::Identifier),
    #[fail(
        display = "trait impl for {:?} does not meet well-formedness requirements",
        _0
    )]
    IllFormedTraitImpl(chalk_ir::Identifier),
}

pub struct WfSolver<'me> {
    pub program: &'me dyn RustIrSource,
    pub env: Arc<ProgramEnvironment>,
    pub solver_choice: SolverChoice,
}

/// A trait for retrieving all types appearing in some Chalk construction.
trait FoldInputTypes {
    fn fold(&self, accumulator: &mut Vec<Ty>);
}

impl<T: FoldInputTypes> FoldInputTypes for Vec<T> {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        for f in self {
            f.fold(accumulator);
        }
    }
}

impl FoldInputTypes for Parameter {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        if let ParameterKind::Ty(ty) = &self.0 {
            ty.fold(accumulator)
        }
    }
}

impl FoldInputTypes for Ty {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        match self {
            Ty::Apply(app) => {
                accumulator.push(self.clone());
                app.parameters.fold(accumulator);
            }
            Ty::Projection(proj) => {
                accumulator.push(self.clone());
                proj.parameters.fold(accumulator);
            }
            Ty::UnselectedProjection(proj) => {
                accumulator.push(self.clone());
                proj.parameters.fold(accumulator);
            }

            // Type parameters do not carry any input types (so we can sort of assume they are
            // always WF).
            Ty::BoundVar(..) => (),

            // Higher-kinded types such as `for<'a> fn(&'a u32)` introduce their own implied
            // bounds, and these bounds will be enforced upon calling such a function. In some
            // sense, well-formedness requirements for the input types of an HKT will be enforced
            // lazily, so no need to include them here.
            Ty::ForAll(..) => (),

            Ty::InferenceVar(..) => {
                panic!("unexpected inference variable in wf rules: {:?}", self,)
            }
        }
    }
}

impl FoldInputTypes for TraitRef {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        self.parameters.fold(accumulator);
    }
}

impl FoldInputTypes for ProjectionEq {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        Ty::Projection(self.projection.clone()).fold(accumulator);
        self.ty.fold(accumulator);
    }
}

impl FoldInputTypes for WhereClause {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        match self {
            WhereClause::Implemented(tr) => tr.fold(accumulator),
            WhereClause::ProjectionEq(p) => p.fold(accumulator),
        }
    }
}

impl<T: FoldInputTypes> FoldInputTypes for Binders<T> {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        self.value.fold(accumulator);
    }
}

impl<'me> WfSolver<'me> {
    pub fn verify_struct_decl(&self, struct_datum: &StructDatum) -> bool {
        // We retrieve all the input types of the struct fields.
        let mut input_types = Vec::new();
        struct_datum.binders.value.fields.fold(&mut input_types);
        struct_datum
            .binders
            .value
            .where_clauses
            .fold(&mut input_types);

        if input_types.is_empty() {
            return true;
        }

        let goals = input_types
            .into_iter()
            .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)))
            .casted();
        let goal = goals
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("at least one goal");

        let hypotheses = struct_datum
            .binders
            .value
            .where_clauses
            .iter()
            .cloned()
            .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
            .casted()
            .collect();

        // We ask that the above input types are well-formed provided that all the where-clauses
        // on the struct definition hold.
        let goal = Goal::Implies(hypotheses, Box::new(goal))
            .quantify(QuantifierKind::ForAll, struct_datum.binders.binders.clone());

        match self
            .solver_choice
            .solve_root_goal(&*self.env, &goal.into_closed_goal())
            .unwrap()
        {
            Some(sol) => sol.is_unique(),
            None => false,
        }
    }

    pub fn verify_trait_impl(&self, impl_datum: &ImplDatum) -> bool {
        let trait_ref = match impl_datum.binders.value.trait_ref {
            PolarizedTraitRef::Positive(ref trait_ref) => trait_ref,
            _ => return true,
        };

        // We retrieve all the input types of the where clauses appearing on the trait impl,
        // e.g. in:
        // ```
        // impl<T, K> Foo for (T, K) where T: Iterator<Item = (HashSet<K>, Vec<Box<T>>)> { ... }
        // ```
        // we would retrieve `HashSet<K>`, `Box<T>`, `Vec<Box<T>>`, `(HashSet<K>, Vec<Box<T>>)`.
        // We will have to prove that these types are well-formed (e.g. an additional `K: Hash`
        // bound would be needed here).
        let mut input_types = Vec::new();
        impl_datum
            .binders
            .value
            .where_clauses
            .fold(&mut input_types);

        // We retrieve all the input types of the type on which we implement the trait: we will
        // *assume* that these types are well-formed, e.g. we will be able to derive that
        // `K: Hash` holds without writing any where clause.
        //
        // Example:
        // ```
        // struct HashSet<K> where K: Hash { ... }
        //
        // impl<K> Foo for HashSet<K> {
        //     // Inside here, we can rely on the fact that `K: Hash` holds
        // }
        // ```
        let mut header_input_types = Vec::new();
        trait_ref.fold(&mut header_input_types);

        // Associated type values are special because they can be parametric (independently of
        // the impl), so we issue a special goal which is quantified using the binders of the
        // associated type value, for example in:
        // ```
        // trait Foo {
        //     type Item<'a>
        // }
        //
        // impl<T> Foo for Box<T> {
        //     type Item<'a> = Box<&'a T>;
        // }
        // ```
        // we would issue the following subgoal: `forall<'a> { WellFormed(Box<&'a T>) }`.
        let compute_assoc_ty_goal = |assoc_ty: &AssociatedTyValue| {
            let assoc_ty_datum = self.program.associated_ty_data(assoc_ty.associated_ty_id);
            let bounds = &assoc_ty_datum.bounds;

            let mut input_types = Vec::new();
            assoc_ty.value.value.ty.fold(&mut input_types);

            let wf_goals = input_types
                .into_iter()
                .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)))
                .casted();

            let trait_ref = trait_ref.shifted_in(assoc_ty.value.binders.len());

            let all_parameters: Vec<_> = assoc_ty
                .value
                .binders
                .iter()
                .zip(0..)
                .map(|p| p.to_parameter())
                .chain(trait_ref.parameters.iter().cloned())
                .collect();

            // Add bounds from the trait. Because they are defined on the trait,
            // their parameters must be substituted with those of the impl.
            let bound_goals = bounds
                .iter()
                .map(|b| Subst::apply(&all_parameters, b))
                .flat_map(|b| b.into_where_clauses(assoc_ty.value.value.ty.clone()))
                .map(|wc| wc.map(|bound| bound.into_well_formed_goal()))
                .casted();

            let goals = wf_goals.chain(bound_goals);
            let goal = match goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf))) {
                Some(goal) => goal,
                None => return None,
            };

            // Add where clauses from the associated ty definition. We must
            // substitute parameters here, like we did with the bounds above.
            let hypotheses = assoc_ty_datum
                .where_clauses
                .iter()
                .map(|wc| Subst::apply(&all_parameters, wc))
                .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
                .casted()
                .collect();

            let goal = Goal::Implies(hypotheses, Box::new(goal));

            Some(goal.quantify(QuantifierKind::ForAll, assoc_ty.value.binders.clone()))
        };

        let assoc_ty_goals = impl_datum
            .binders
            .value
            .associated_ty_values
            .iter()
            .filter_map(compute_assoc_ty_goal);

        // Things to prove well-formed: input types of the where-clauses, projection types
        // appearing in the header, associated type values, and of course the trait ref.
        let trait_ref_wf = DomainGoal::WellFormed(WellFormed::Trait(trait_ref.clone()));
        let goals = input_types
            .into_iter()
            .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)).cast())
            .chain(assoc_ty_goals)
            .chain(Some(trait_ref_wf).cast());

        let goal = goals
            .fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
            .expect("at least one goal");

        // Assumptions: types appearing in the header which are not projection types are
        // assumed to be well-formed, and where clauses declared on the impl are assumed
        // to hold.
        let hypotheses = impl_datum
            .binders
            .value
            .where_clauses
            .iter()
            .cloned()
            .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
            .casted()
            .chain(
                header_input_types
                    .into_iter()
                    .map(|ty| DomainGoal::FromEnv(FromEnv::Ty(ty)))
                    .casted(),
            )
            .collect();

        let goal = Goal::Implies(hypotheses, Box::new(goal))
            .quantify(QuantifierKind::ForAll, impl_datum.binders.binders.clone());

        debug!("WF trait goal: {:?}", goal);

        match self
            .solver_choice
            .solve_root_goal(&*self.env, &goal.into_closed_goal())
            .unwrap()
        {
            Some(sol) => sol.is_unique(),
            None => false,
        }
    }
}
