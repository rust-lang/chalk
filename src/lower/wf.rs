use std::sync::Arc;

use ir::*;
use errors::*;
use cast::Cast;
use solve::SolverChoice;
use itertools::Itertools;

struct WfSolver {
    env: Arc<ProgramEnvironment>,
    solver_choice: SolverChoice,
}

impl Program {
    pub fn verify_well_formedness(&self, solver_choice: SolverChoice) -> Result<()> {
        set_current_program(&Arc::new(self.clone()), || self.solve_wf_requirements(solver_choice))
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
        match *self {
            ParameterKind::Ty(ref ty) => ty.fold(accumulator),
            _ => (),
        }
    }
}

impl FoldInputTypes for Ty {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        match *self {
            Ty::Apply(ref app) => {
                accumulator.push(self.clone());
                app.parameters.fold(accumulator);
            }
            Ty::Projection(ref proj) => {
                accumulator.push(self.clone());
                proj.parameters.fold(accumulator);
            }
            Ty::UnselectedProjection(ref proj) => {
                accumulator.push(self.clone());
                proj.parameters.fold(accumulator);
            }
            _ => (),
        }
    }
}

impl FoldInputTypes for TraitRef {
    fn fold(&self, accumulator: &mut Vec<Ty>) {
        self.parameters.fold(accumulator);
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
        match *self {
            DomainGoal::Implemented(ref tr) => tr.fold(accumulator),
            DomainGoal::Normalize(ref n) => n.fold(accumulator),
            DomainGoal::UnselectedNormalize(ref n) => n.fold(accumulator),
            _ => (),
        }
    }
}

impl WfSolver {
    fn verify_struct_decl(&self, struct_datum: &StructDatum) -> bool {
        let mut input_types = Vec::new();
        struct_datum.binders.value.fields.fold(&mut input_types);
        struct_datum.binders.value.where_clauses.fold(&mut input_types);

        if input_types.is_empty() {
            return true;
        }

        let goals = input_types.into_iter().map(|ty| WellFormed::Ty(ty).cast());

        let goal = goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                        .expect("at least one goal");

        let hypotheses =
            struct_datum.binders
                        .value
                        .where_clauses
                        .iter()
                        .cloned()
                        .map(|wc| wc.into_from_env_clause())
                        .collect();

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

        let mut input_types = Vec::new();
        impl_datum.binders.value.where_clauses.fold(&mut input_types);

        let compute_assoc_ty_goal = |assoc_ty: &AssociatedTyValue| {
            let mut input_types = Vec::new();
            assoc_ty.value.value.ty.fold(&mut input_types);

            if input_types.is_empty() {
                return None;
            }

            let goals = input_types.into_iter().map(|ty| WellFormed::Ty(ty).cast());
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

        let goals =
            input_types.into_iter()
                       .map(|ty| WellFormed::Ty(ty).cast())
                       .chain(assoc_ty_goals)
                       .chain(Some(WellFormed::TraitRef(trait_ref.clone())).cast());
        
        let goal = goals.fold1(|goal, leaf| Goal::And(Box::new(goal), Box::new(leaf)))
                        .expect("at least one goal");

        let mut input_types = Vec::new();
        trait_ref.fold(&mut input_types);

        let hypotheses =
            impl_datum.binders
                      .value
                      .where_clauses
                      .iter()
                      .cloned()
                      .map(|wc| wc.into_from_env_clause())
                      .chain(input_types.into_iter().map(|ty| FromEnv::Ty(ty).cast()))
                      .collect();

        let goal = Goal::Implies(hypotheses, Box::new(goal))
            .quantify(QuantifierKind::ForAll, impl_datum.binders.binders.clone());

        match self.solver_choice.solve_root_goal(&self.env, &goal.into_closed_goal()).unwrap() {
            Some(sol) => sol.is_unique(),
            None => false,
        }
    }
}
