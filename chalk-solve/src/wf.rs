use std::fmt;

use crate::ext::*;
use crate::solve::SolverChoice;
use crate::split::Split;
use crate::RustIrDatabase;
use chalk_ir::cast::*;
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::*;
use chalk_rust_ir::*;

#[derive(Debug)]
pub enum WfError<I: Interner> {
    IllFormedTypeDecl(chalk_ir::StructId<I>),
    IllFormedTraitImpl(chalk_ir::TraitId<I>),
}

impl<I: Interner> fmt::Display for WfError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WfError::IllFormedTypeDecl(id) => write!(
                f,
                "type declaration `{:?}` does not meet well-formedness requirements",
                id
            ),
            WfError::IllFormedTraitImpl(id) => write!(
                f,
                "trait impl for `{:?}` does not meet well-formedness requirements",
                id
            ),
        }
    }
}

impl<I: Interner> std::error::Error for WfError<I> {}

pub struct WfSolver<'db, I: Interner> {
    db: &'db dyn RustIrDatabase<I>,
    solver_choice: SolverChoice,
}

/// A trait for retrieving all types appearing in some Chalk construction.
trait FoldInputTypes: HasInterner {
    fn fold(&self, interner: &Self::Interner, accumulator: &mut Vec<Ty<Self::Interner>>);
}

impl<T: FoldInputTypes> FoldInputTypes for [T] {
    fn fold(&self, interner: &T::Interner, accumulator: &mut Vec<Ty<T::Interner>>) {
        for f in self {
            f.fold(interner, accumulator);
        }
    }
}

impl<T: FoldInputTypes> FoldInputTypes for Vec<T> {
    fn fold(&self, interner: &T::Interner, accumulator: &mut Vec<Ty<T::Interner>>) {
        for f in self {
            f.fold(interner, accumulator);
        }
    }
}

impl<I: Interner> FoldInputTypes for Parameter<I> {
    fn fold(&self, interner: &I, accumulator: &mut Vec<Ty<I>>) {
        if let ParameterKind::Ty(ty) = self.data() {
            ty.fold(interner, accumulator)
        }
    }
}

impl<I: Interner> FoldInputTypes for Substitution<I> {
    fn fold(&self, interner: &I, accumulator: &mut Vec<Ty<I>>) {
        self.parameters().fold(interner, accumulator)
    }
}

impl<I: Interner> FoldInputTypes for Ty<I> {
    fn fold(&self, interner: &I, accumulator: &mut Vec<Ty<I>>) {
        match self.data(interner) {
            TyData::Apply(app) => {
                accumulator.push(self.clone());
                app.substitution.fold(interner, accumulator);
            }

            TyData::Dyn(qwc) => {
                accumulator.push(self.clone());
                qwc.bounds.fold(interner, accumulator);
            }

            TyData::Alias(alias) => {
                accumulator.push(self.clone());
                alias.substitution.fold(interner, accumulator);
            }

            TyData::Placeholder(_) => {
                accumulator.push(self.clone());
            }

            // Type parameters do not carry any input types (so we can sort of assume they are
            // always WF).
            TyData::BoundVar(..) => (),

            // Higher-kinded types such as `for<'a> fn(&'a u32)` introduce their own implied
            // bounds, and these bounds will be enforced upon calling such a function. In some
            // sense, well-formedness requirements for the input types of an HKT will be enforced
            // lazily, so no need to include them here.
            TyData::Function(..) => (),

            TyData::InferenceVar(..) => {
                panic!("unexpected inference variable in wf rules: {:?}", self)
            }
        }
    }
}

impl<I: Interner> FoldInputTypes for TraitRef<I> {
    fn fold(&self, interner: &I, accumulator: &mut Vec<Ty<I>>) {
        self.substitution.fold(interner, accumulator);
    }
}

impl<I: Interner> FoldInputTypes for AliasEq<I> {
    fn fold(&self, interner: &I, accumulator: &mut Vec<Ty<I>>) {
        TyData::Alias(self.alias.clone())
            .intern(interner)
            .fold(interner, accumulator);
        self.ty.fold(interner, accumulator);
    }
}

impl<I: Interner> FoldInputTypes for WhereClause<I> {
    fn fold(&self, interner: &I, accumulator: &mut Vec<Ty<I>>) {
        match self {
            WhereClause::Implemented(tr) => tr.fold(interner, accumulator),
            WhereClause::AliasEq(p) => p.fold(interner, accumulator),
        }
    }
}

impl<T: FoldInputTypes> FoldInputTypes for Binders<T> {
    fn fold(&self, interner: &T::Interner, accumulator: &mut Vec<Ty<T::Interner>>) {
        self.value.fold(interner, accumulator);
    }
}

impl<'db, I> WfSolver<'db, I>
where
    I: Interner,
{
    /// Constructs a new `WfSolver`.
    pub fn new(db: &'db dyn RustIrDatabase<I>, solver_choice: SolverChoice) -> Self {
        Self { db, solver_choice }
    }

    pub fn verify_struct_decl(&self, struct_id: StructId<I>) -> Result<(), WfError<I>> {
        let interner = self.db.interner();
        let struct_datum = self.db.struct_datum(struct_id);

        // We retrieve all the input types of the struct fields.
        let mut input_types = Vec::new();
        struct_datum
            .binders
            .value
            .fields
            .fold(interner, &mut input_types);
        struct_datum
            .binders
            .value
            .where_clauses
            .fold(interner, &mut input_types);

        if input_types.is_empty() {
            return Ok(());
        }

        let goals = input_types
            .into_iter()
            .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)))
            .casted(interner);
        let goal = Goal::all(interner, goals);

        let hypotheses = struct_datum
            .binders
            .value
            .where_clauses
            .iter()
            .cloned()
            .map(|wc| wc.map(|bound| bound.into_from_env_goal(interner)))
            .casted(interner)
            .collect();

        // We ask that the above input types are well-formed provided that all the where-clauses
        // on the struct definition hold.
        let goal = GoalData::Implies(hypotheses, goal)
            .intern(interner)
            .quantify(
                interner,
                QuantifierKind::ForAll,
                struct_datum.binders.binders.clone(),
            );

        let is_legal = match self
            .solver_choice
            .into_solver()
            .solve(self.db, &goal.into_closed_goal(interner))
        {
            Some(sol) => sol.is_unique(),
            None => false,
        };

        if !is_legal {
            Err(WfError::IllFormedTypeDecl(struct_id))
        } else {
            Ok(())
        }
    }

    pub fn verify_trait_impl(&self, impl_id: ImplId<I>) -> Result<(), WfError<I>> {
        let interner = self.db.interner();
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
            .fold(interner, &mut input_types);

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
        trait_ref.fold(interner, &mut header_input_types);

        let assoc_ty_goals = impl_datum
            .associated_ty_value_ids
            .iter()
            .filter_map(|&id| self.compute_assoc_ty_goal(id));

        // Things to prove well-formed: input types of the where-clauses, projection types
        // appearing in the header, associated type values, and of course the trait ref.
        let trait_ref_wf = DomainGoal::WellFormed(WellFormed::Trait(trait_ref.clone()));
        let goals = input_types
            .into_iter()
            .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)).cast(interner))
            .chain(assoc_ty_goals)
            .chain(Some(trait_ref_wf).cast(interner));

        let goal = Goal::all(interner, goals);

        // Assumptions: types appearing in the header which are not projection types are
        // assumed to be well-formed, and where clauses declared on the impl are assumed
        // to hold.
        let hypotheses = impl_datum
            .binders
            .value
            .where_clauses
            .iter()
            .cloned()
            .map(|qwc| qwc.into_from_env_goal(interner))
            .casted(interner)
            .chain(
                header_input_types
                    .into_iter()
                    .map(|ty| DomainGoal::FromEnv(FromEnv::Ty(ty)))
                    .casted(interner),
            )
            .collect();

        let goal = GoalData::Implies(hypotheses, goal)
            .intern(interner)
            .quantify(
                interner,
                QuantifierKind::ForAll,
                impl_datum.binders.binders.clone(),
            );

        debug!("WF trait goal: {:?}", goal);

        let is_legal = match self
            .solver_choice
            .into_solver()
            .solve(self.db, &goal.into_closed_goal(interner))
        {
            Some(sol) => sol.is_unique(),
            None => false,
        };

        if is_legal {
            Ok(())
        } else {
            let trait_ref = &impl_datum.binders.value.trait_ref;
            Err(WfError::IllFormedTraitImpl(trait_ref.trait_id))
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
    fn compute_assoc_ty_goal(&self, assoc_ty_id: AssociatedTyValueId<I>) -> Option<Goal<I>> {
        let interner = self.db.interner();
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
            .map(|p| p.to_parameter(interner))
            .collect();

        // Get the projection for this associated type:
        //
        // * `projection`: `<Box<!T> as Foo>::Item<'!a>`
        let (_, projection) = self
            .db
            .impl_parameters_and_projection_from_associated_ty_value(&all_parameters, assoc_ty);

        // Get the ty that the impl is using -- `Box<&'!a !T>`, in our example
        let AssociatedTyValueBound { ty: value_ty } =
            assoc_ty.value.substitute(interner, &all_parameters);

        let mut input_types = Vec::new();
        value_ty.fold(interner, &mut input_types);

        // We require that `WellFormed(T)` for each type that appears in the value
        let wf_goals = input_types
            .into_iter()
            .map(|ty| DomainGoal::WellFormed(WellFormed::Ty(ty)))
            .casted(interner);

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
        } = assoc_ty_datum
            .binders
            .substitute(interner, &projection.substitution);

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
            .flat_map(|qb| qb.into_where_clauses(interner, value_ty.clone()))
            .map(|qwc| qwc.into_well_formed_goal(interner))
            .casted(interner);

        // Concatenate the WF goals of inner types + the requirements from trait
        let goals = wf_goals.chain(bound_goals);
        let goal = Goal::all(interner, goals);
        if goal.is_trivially_true() {
            return None;
        }

        // Add where clauses from the associated ty definition. We must
        // substitute parameters here, like we did with the bounds above.
        let hypotheses = defn_where_clauses
            .iter()
            .cloned()
            .map(|qwc| qwc.into_from_env_goal(interner))
            .casted(interner)
            .collect();

        let goal = GoalData::Implies(hypotheses, goal).intern(interner);

        // Create a composed goal that is universally quantified over
        // the parameters from the associated type value (e.g.,
        // `forall<'a> { .. }` in our example).
        let (_, value_binders) = self
            .db
            .split_associated_ty_value_parameters(&assoc_ty.value.binders, assoc_ty);

        Some(goal.quantify(interner, QuantifierKind::ForAll, value_binders.to_vec()))
    }
}
