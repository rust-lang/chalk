use std::sync::Arc;

use ir::*;
use errors::*;
use cast::*;
use solve::SolverChoice;
use itertools::Itertools;

struct WfSolver {
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
}

impl Program {
    pub fn verify_well_formedness(&self, solver_choice: SolverChoice) -> Result<()> {
        tls::set_current_program(&Arc::new(self.clone()), || self.solve_wf_requirements(solver_choice))
    }

    fn solve_wf_requirements(&self, solver_choice: SolverChoice) -> Result<()> {
        let solver = WfSolver {
            env: Arc::new(self.environment()),
            solver_choice,
        };

        for (id, struct_datum) in &self.struct_data {
            if !solver.verify_struct_decl(struct_datum) {
                let name = self.type_kinds.get(id).unwrap().name;
                return Err(Error::from_kind(ErrorKind::IllFormedTypeDecl(name)));
            }
        }

        for impl_datum in self.impl_data.values() {
            if !solver.verify_trait_impl(impl_datum) {
                let trait_ref = impl_datum.binders.value.trait_ref.trait_ref();
                let name = self.type_kinds.get(&trait_ref.trait_id).unwrap().name;
                return Err(Error::from_kind(ErrorKind::IllFormedTraitImpl(name)));
            }
        }

        Ok(())
    }
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
        match self {
            ParameterKind::Ty(ty) => ty.fold(accumulator),
            _ => (),
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
            Ty::Var(..) => (),

            // Higher-kinded types such as `for<'a> fn(&'a u32)` introduce their own implied
            // bounds, and these bounds will be enforced upon calling such a function. In some
            // sense, well-formedness requirements for the input types of an HKT will be enforced
            // lazily, so no need to include them here.
            Ty::ForAll(..) => (),
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

impl FoldInputTypes for WhereClauseAtom {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        match self {
            WhereClauseAtom::Implemented(tr) => tr.fold(accumulator),
            WhereClauseAtom::ProjectionEq(p) => p.fold(accumulator),
        }
    }
}

impl FoldInputTypes for Normalize {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        self.projection.parameters.fold(accumulator);
        self.ty.fold(accumulator);
    }
}

impl FoldInputTypes for UnselectedNormalize {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        self.projection.parameters.fold(accumulator);
        self.ty.fold(accumulator);
    }
}

impl FoldInputTypes for DomainGoal {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        match self {
            DomainGoal::Holds(wca) => wca.fold(accumulator),
            DomainGoal::Normalize(n) => n.fold(accumulator),
            DomainGoal::UnselectedNormalize(n) => n.fold(accumulator),

            DomainGoal::WellFormed(..) |
            DomainGoal::FromEnv(..) |
            DomainGoal::WellFormedTy(..) |
            DomainGoal::FromEnvTy(..) |
            DomainGoal::Derefs(..) => panic!("unexpected where clause"),

            DomainGoal::InScope(..) => (),
        }
    }
}

impl<T: FoldInputTypes> FoldInputTypes for Binders<T> {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        self.value.fold(accumulator);
    }
}

impl WfSolver {
    fn verify_struct_decl(&self, struct_datum: &StructDatum) -> bool {
        // We retrieve all the input types of the struct fields.
        let mut input_types = Vec::new();
        struct_datum.binders.value.fields.fold(&mut input_types);
        struct_datum.binders.value.where_clauses.fold(&mut input_types);

        if input_types.is_empty() {
            return true;
        }

        let goals = input_types.into_iter().map(|ty| DomainGoal::WellFormedTy(ty).cast());
        let goal = goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                        .expect("at least one goal");

        let hypotheses =
            struct_datum.binders
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

        match self.solver_choice.solve_root_goal(&self.env, &goal.into_closed_goal()).unwrap() {
            Some(sol) => sol.is_unique(),
            None => false,
        }
    }

    fn verify_trait_impl(&self, impl_datum: &ImplDatum) -> bool {
        let trait_ref = match impl_datum.binders.value.trait_ref {
            PolarizedTraitRef::Positive(ref trait_ref) => trait_ref,
            _ => return true
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
        impl_datum.binders.value.where_clauses.fold(&mut input_types);

        // We partition the input types of the type on which we implement the trait in two categories:
        // * projection types, e.g. `<T as Iterator>::Item`: we will have to prove that these types
        //   are well-formed, e.g. that we can show that `T: Iterator` holds
        // * any other types, e.g. `HashSet<K>`: we will *assume* that these types are well-formed, e.g.
        //   we will be able to derive that `K: Hash` holds without writing any where clause.
        //
        // Examples:
        // ```
        // struct HashSet<K> where K: Hash { ... }
        //
        // impl<K> Foo for HashSet<K> {
        //     // Inside here, we can rely on the fact that `K: Hash` holds
        // }
        // ```
        //
        // ```
        // impl<T> Foo for <T as Iterator>::Item {
        //     // The impl is not well-formed, as an exception we do not assume that
        //     // `<T as Iterator>::Item` is well-formed and instead want to prove it.
        // }
        // ```
        //
        // ```
        // impl<T> Foo for <T as Iterator>::Item where T: Iterator {
        //     // Now ok.
        // }
        // ```
        let mut header_input_types = Vec::new();
        trait_ref.fold(&mut header_input_types);
        let (header_projection_types, header_other_types): (Vec<_>, Vec<_>) =
            header_input_types.into_iter()
                              .partition(|ty| ty.is_projection());

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
            let mut input_types = Vec::new();
            assoc_ty.value.value.ty.fold(&mut input_types);

            if input_types.is_empty() {
                return None;
            }

            let goals = input_types.into_iter().map(|ty| DomainGoal::WellFormedTy(ty).cast());
            let goal = goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                            .expect("at least one goal");
            Some(goal.quantify(QuantifierKind::ForAll, assoc_ty.value.binders.clone()))
        };

        let assoc_ty_goals =
            impl_datum.binders
                      .value
                      .associated_ty_values
                      .iter()
                      .filter_map(compute_assoc_ty_goal);

        // Things to prove well-formed: input types of the where-clauses, projection types
        // appearing in the header, associated type values, and of course the trait ref.
        let trait_ref_wf = DomainGoal::WellFormed(
            WhereClauseAtom::Implemented(trait_ref.clone())
        );
        let goals =
            input_types.into_iter()
                       .chain(header_projection_types.into_iter())
                       .map(|ty| DomainGoal::WellFormedTy(ty).cast())
                       .chain(assoc_ty_goals)
                       .chain(Some(trait_ref_wf).cast());

        let goal = goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                        .expect("at least one goal");

        // Assumptions: types appearing in the header which are not projection types are
        // assumed to be well-formed, and where clauses declared on the impl are assumed
        // to hold.
        let hypotheses =
            impl_datum.binders
                      .value
                      .where_clauses
                      .iter()
                      .cloned()
                      .map(|wc| wc.map(|bound| bound.into_from_env_goal()))
                      .casted()
                      .chain(header_other_types.into_iter().map(|ty| DomainGoal::FromEnvTy(ty).cast()))
                      .collect();

        let goal = Goal::Implies(hypotheses, Box::new(goal))
            .quantify(QuantifierKind::ForAll, impl_datum.binders.binders.clone());

        match self.solver_choice.solve_root_goal(&self.env, &goal.into_closed_goal()).unwrap() {
            Some(sol) => sol.is_unique(),
            None => false,
        }
    }
}
