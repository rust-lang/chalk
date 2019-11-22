use std::fmt;

use crate::ext::*;
use crate::solve::SolverChoice;
use crate::split::Split;
use crate::RustIrDatabase;
use chalk_ir::cast::*;
use chalk_ir::family::{HasTypeFamily, TypeFamily};
use chalk_ir::*;
use chalk_rust_ir::*;
use itertools::Itertools;

#[derive(Debug)]
pub enum WfError {
    IllFormedTypeDecl(chalk_ir::Identifier),
    IllFormedTraitImpl(chalk_ir::Identifier),
}

impl fmt::Display for WfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WfError::IllFormedTypeDecl(id) => write!(
                f,
                "type declaration \"{}\" does not meet well-formedness requirements",
                id
            ),
            WfError::IllFormedTraitImpl(id) => write!(
                f,
                "trait impl for \"{}\" does not meet well-formedness requirements",
                id
            ),
        }
    }
}

impl std::error::Error for WfError {}

pub struct WfSolver<'db, TF: TypeFamily> {
    db: &'db dyn RustIrDatabase<TF>,
    solver_choice: SolverChoice,
}

/// A trait for retrieving all types appearing in some Chalk construction.
trait FoldInputTypes: HasTypeFamily {
    fn fold(&self, accumulator: &mut Vec<Ty<Self::TypeFamily>>);
}

impl<T: FoldInputTypes> FoldInputTypes for Vec<T> {
    fn fold(&self, accumulator: &mut Vec<Ty<T::TypeFamily>>) {
        for f in self {
            f.fold(accumulator);
        }
    }
}

impl<TF: TypeFamily> FoldInputTypes for Parameter<TF> {
    fn fold(&self, accumulator: &mut Vec<Ty<TF>>) {
        if let ParameterKind::Ty(ty) = &self.0 {
            ty.fold(accumulator)
        }
    }
}

impl<TF: TypeFamily> FoldInputTypes for Ty<TF> {
    fn fold(&self, accumulator: &mut Vec<Ty<TF>>) {
        match self.data() {
            TyData::Apply(app) => {
                accumulator.push(self.clone());
                app.parameters.fold(accumulator);
            }
            TyData::Dyn(qwc) | TyData::Opaque(qwc) => {
                accumulator.push(self.clone());
                qwc.fold(accumulator);
            }
            TyData::Projection(proj) => {
                accumulator.push(self.clone());
                proj.parameters.fold(accumulator);
            }

            // Type parameters do not carry any input types (so we can sort of assume they are
            // always WF).
            TyData::BoundVar(..) => (),

            // Higher-kinded types such as `for<'a> fn(&'a u32)` introduce their own implied
            // bounds, and these bounds will be enforced upon calling such a function. In some
            // sense, well-formedness requirements for the input types of an HKT will be enforced
            // lazily, so no need to include them here.
            TyData::ForAll(..) => (),

            TyData::InferenceVar(..) => {
                panic!("unexpected inference variable in wf rules: {:?}", self)
            }
        }
    }
}

impl<TF: TypeFamily> FoldInputTypes for TraitRef<TF> {
    fn fold(&self, accumulator: &mut Vec<Ty<TF>>) {
        self.parameters.fold(accumulator);
    }
}

impl<TF: TypeFamily> FoldInputTypes for ProjectionEq<TF> {
    fn fold(&self, accumulator: &mut Vec<Ty<TF>>) {
        TyData::Projection(self.projection.clone())
            .intern()
            .fold(accumulator);
        self.ty.fold(accumulator);
    }
}

impl<TF: TypeFamily> FoldInputTypes for WhereClause<TF> {
    fn fold(&self, accumulator: &mut Vec<Ty<TF>>) {
        match self {
            WhereClause::Implemented(tr) => tr.fold(accumulator),
            WhereClause::ProjectionEq(p) => p.fold(accumulator),
        }
    }
}

impl<T: FoldInputTypes> FoldInputTypes for Binders<T> {
    fn fold(&self, accumulator: &mut Vec<Ty<T::TypeFamily>>) {
        self.value.fold(accumulator);
    }
}

impl<'db, TF> WfSolver<'db, TF>
where
    TF: TypeFamily,
{
    /// Constructs a new `WfSolver`.
    pub fn new(db: &'db dyn RustIrDatabase<TF>, solver_choice: SolverChoice) -> Self {
        Self { db, solver_choice }
    }

    pub fn verify_struct_decl(&self, struct_id: StructId) -> Result<(), WfError> {
        let struct_datum = self.db.struct_datum(struct_id);

        // We retrieve all the input types of the struct fields.
        let mut input_types = Vec::new();
        struct_datum.binders.value.fields.fold(&mut input_types);
        struct_datum
            .binders
            .value
            .where_clauses
            .fold(&mut input_types);

        if input_types.is_empty() {
            return Ok(());
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

        let is_legal = match self
            .solver_choice
            .into_solver()
            .solve(self.db, &goal.into_closed_goal())
        {
            Some(sol) => sol.is_unique(),
            None => false,
        };

        if !is_legal {
            let name = self.db.type_name(struct_id.into());
            Err(WfError::IllFormedTypeDecl(name))
        } else {
            Ok(())
        }
    }

    pub fn verify_trait_impl(&self, impl_id: ImplId) -> Result<(), WfError> {
        let impl_datum = self.db.impl_datum(impl_id);

        if !impl_datum.is_positive() {
            return Ok(());
        }

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
        let trait_ref = &impl_datum.binders.value.trait_ref;
        trait_ref.fold(&mut header_input_types);

        let assoc_ty_goals = impl_datum
            .associated_ty_value_ids
            .iter()
            .filter_map(|&id| self.compute_assoc_ty_goal(id));

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
            .map(|qwc| qwc.into_from_env_goal())
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

        let is_legal = match self
            .solver_choice
            .into_solver()
            .solve(self.db, &goal.into_closed_goal())
        {
            Some(sol) => sol.is_unique(),
            None => false,
        };

        if is_legal {
            Ok(())
        } else {
            let trait_ref = &impl_datum.binders.value.trait_ref;
            let name = self.db.type_name(trait_ref.trait_id.into());
            Err(WfError::IllFormedTraitImpl(name))
        }
    }

    /// Associated type values are special because they can be parametric (independently of
    /// the impl), so we issue a special goal which is quantified using the binders of the
    /// associated type value, for example in:
    ///
    /// ```ignore
    /// trait Foo {
    ///     type Item<'a>: Clone where Self: 'a
    /// }
    ///
    /// impl<T> Foo for Box<T> {
    ///     type Item<'a> = Box<&'a T>;
    /// }
    /// ```
    ///
    /// we would issue the following subgoal: `forall<'a> { WellFormed(Box<&'a T>) }`.
    ///
    /// Note that there is no binder for `T` in the above: the goal we
    /// generate is expected to be exected in the context of the
    /// larger WF goal for the impl, which already has such a
    /// binder. So the entire goal for the impl might be:
    ///
    /// ```ignore
    /// forall<T> {
    ///     WellFormed(Box<T>) /* this comes from the impl, not this routine */,
    ///     forall<'a> { WellFormed(Box<&'a T>) },
    /// }
    /// ```
    fn compute_assoc_ty_goal(&self, assoc_ty_id: AssociatedTyValueId) -> Option<Goal<TF>> {
        let assoc_ty = &self.db.associated_ty_value(assoc_ty_id);

        // The substitutions for the binders on this associated type
        // value. These would be placeholders like `'!a` and `!T`, in
        // our example above.
        //
        // We begin with the associated type binders, as that `forall`
        // is innermost, and then chain in the binders from the impl
        // (which are generated by the caller).
        let all_parameters: Vec<_> = assoc_ty
            .value
            .binders
            .iter()
            .zip(0..)
            .map(|p| p.to_parameter())
            .collect();

        // Get the projection for this associated type:
        //
        // * `projection`: `<Box<!T> as Foo>::Item<'!a>`
        let (_, projection) = self
            .db
            .impl_parameters_and_projection_from_associated_ty_value(&all_parameters, assoc_ty);

        // Get the ty that the impl is using -- `Box<&'!a !T>`, in our example
        let AssociatedTyValueBound { ty: value_ty } = assoc_ty.value.substitute(&all_parameters);

        let mut input_types = Vec::new();
        value_ty.fold(&mut input_types);

        // We require that `WellFormed(T)` for each type that appears in the value
        let wf_goals = input_types
            .into_iter()
            .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)))
            .casted();

        // Get the bounds and where clauses from the trait
        // declaration, substituted appropriately.
        //
        // From our example:
        //
        // * bounds
        //     * original in trait, `Clone`
        //     * after substituting impl parameters, `Clone`
        //     * note that the self-type is not yet supplied for bounds,
        //       we will do that later
        // * where clauses
        //     * original in trait, `Self: 'a`
        //     * after substituting impl parameters, `Box<!T>: '!a`
        let assoc_ty_datum = self.db.associated_ty_data(projection.associated_ty_id);
        let AssociatedTyDatumBound {
            bounds: defn_bounds,
            where_clauses: defn_where_clauses,
        } = assoc_ty_datum.binders.substitute(&projection.parameters);

        // Check that the `value_ty` meets the bounds from the trait.
        // Here we take the substituted bounds (`defn_bounds`) and we
        // supply the self-type `value_ty` to yield the final result.
        //
        // In our example, the bound was `Clone`, so the combined
        // result is `Box<!T>: Clone`. This is then converted to a
        // well-formed goal like `WellFormed(Box<!T>: Clone)`.
        let bound_goals = defn_bounds
            .iter()
            .cloned()
            .flat_map(|qb| qb.into_where_clauses(value_ty.clone()))
            .map(|qwc| qwc.into_well_formed_goal())
            .casted();

        // Concatenate the WF goals of inner types + the requirements from trait
        let goals = wf_goals.chain(bound_goals);
        let goal = match goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf))) {
            Some(goal) => goal,
            None => return None,
        };

        // Add where clauses from the associated ty definition. We must
        // substitute parameters here, like we did with the bounds above.
        let hypotheses = defn_where_clauses
            .iter()
            .cloned()
            .map(|qwc| qwc.into_from_env_goal())
            .casted()
            .collect();

        let goal = Goal::Implies(hypotheses, Box::new(goal));

        // Create a composed goal that is universally quantified over
        // the parameters from the associated type value (e.g.,
        // `forall<'a> { .. }` in our example).
        let (_, value_binders) = self
            .db
            .split_associated_ty_value_parameters(&assoc_ty.value.binders, assoc_ty);

        Some(goal.quantify(QuantifierKind::ForAll, value_binders.to_vec()))
    }
}
