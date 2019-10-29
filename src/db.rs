use crate::error::ChalkError;
use crate::lowering::LowerGoal;
use crate::program::Program;
use crate::query::{Lowering, LoweringDatabase};
use chalk_ir::could_match::CouldMatch;
use chalk_ir::family::ChalkIr;
use chalk_ir::tls;
use chalk_ir::Canonical;
use chalk_ir::ConstrainedSubst;
use chalk_ir::Goal;
use chalk_ir::Identifier;
use chalk_ir::ImplId;
use chalk_ir::InEnvironment;
use chalk_ir::Parameter;
use chalk_ir::ProgramClause;
use chalk_ir::ProjectionTy;
use chalk_ir::StructId;
use chalk_ir::TraitId;
use chalk_ir::Ty;
use chalk_ir::TypeId;
use chalk_ir::TypeKindId;
use chalk_ir::TypeName;
use chalk_ir::UCanonical;
use chalk_rust_ir::AssociatedTyDatum;
use chalk_rust_ir::ImplDatum;
use chalk_rust_ir::ImplType;
use chalk_rust_ir::StructDatum;
use chalk_rust_ir::TraitDatum;
use chalk_solve::RustIrDatabase;
use chalk_solve::Solution;
use chalk_solve::SolverChoice;
use salsa::Database;
use std::sync::Arc;

#[salsa::database(Lowering)]
#[derive(Debug, Default)]
pub struct ChalkDatabase {
    runtime: salsa::Runtime<ChalkDatabase>,
}

impl Database for ChalkDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime<ChalkDatabase> {
        &self.runtime
    }
}

impl ChalkDatabase {
    pub fn with(program_text: &str, solver_choice: SolverChoice) -> Self {
        let mut db = ChalkDatabase::default();
        db.set_program_text(Arc::new(program_text.to_string()));
        db.set_solver_choice(solver_choice);
        db
    }

    pub fn with_program<R>(&self, op: impl FnOnce(&Program) -> R) -> R {
        let program = &self.checked_program().unwrap();
        tls::set_current_program(&program, || op(&program))
    }

    pub fn parse_and_lower_goal(&self, text: &str) -> Result<Box<Goal<ChalkIr>>, ChalkError> {
        let program = self.checked_program()?;
        Ok(chalk_parse::parse_goal(text)?.lower(&*program)?)
    }

    pub fn solve(&self, goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>) -> Option<Solution> {
        let solver = self.solver();
        let solution = solver.lock().unwrap().solve(self, goal);
        solution
    }

    pub fn solve_multiple(
        &self,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
        f: impl FnMut(Canonical<ConstrainedSubst<ChalkIr>>, bool) -> bool,
    ) -> bool {
        let solver = self.solver();
        let solution = solver.lock().unwrap().solve_multiple(self, goal, f);
        solution
    }
}

impl RustIrDatabase for ChalkDatabase {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        self.program_ir().unwrap().custom_clauses.clone()
    }

    fn associated_ty_data(&self, ty: TypeId) -> Arc<AssociatedTyDatum> {
        self.program_ir().unwrap().associated_ty_data[&ty].clone()
    }

    fn trait_datum(&self, id: TraitId) -> Arc<TraitDatum> {
        self.program_ir().unwrap().trait_data[&id].clone()
    }

    fn impl_datum(&self, id: ImplId) -> Arc<ImplDatum> {
        self.program_ir().unwrap().impl_data[&id].clone()
    }

    fn struct_datum(&self, id: StructId) -> Arc<StructDatum> {
        self.program_ir().unwrap().struct_data[&id].clone()
    }

    fn impls_for_trait(&self, trait_id: TraitId, parameters: &[Parameter<ChalkIr>]) -> Vec<ImplId> {
        self.program_ir()
            .unwrap()
            .impl_data
            .iter()
            .filter(|(_, impl_datum)| {
                let trait_ref = &impl_datum.binders.value.trait_ref;
                trait_id == trait_ref.trait_id && {
                    assert_eq!(trait_ref.parameters.len(), parameters.len());
                    <[_] as CouldMatch<[_]>>::could_match(&parameters, &trait_ref.parameters)
                }
            })
            .map(|(&impl_id, _)| impl_id)
            .collect()
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId) -> Vec<ImplId> {
        self.program_ir()
            .unwrap()
            .impl_data
            .iter()
            .filter(|(_, impl_datum)| {
                impl_datum.trait_id() == trait_id && impl_datum.impl_type == ImplType::Local
            })
            .map(|(&impl_id, _)| impl_id)
            .collect()
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId, struct_id: StructId) -> bool {
        // Look for an impl like `impl Send for Foo` where `Foo` is
        // the struct.  See `push_auto_trait_impls` for more.
        let type_kind_id = TypeKindId::StructId(struct_id);
        self.program_ir()
            .unwrap()
            .impl_data
            .values()
            .any(|impl_datum| {
                let impl_trait_ref = &impl_datum.binders.value.trait_ref;
                impl_trait_ref.trait_id == auto_trait_id
                    && match impl_trait_ref.parameters[0].assert_ty_ref() {
                        Ty::Apply(apply) => match apply.name {
                            TypeName::TypeKindId(id) => id == type_kind_id,
                            _ => false,
                        },

                        _ => false,
                    }
            })
    }

    fn type_name(&self, id: TypeKindId) -> Identifier {
        match self.program_ir().unwrap().type_kinds.get(&id) {
            Some(v) => v.name,
            None => panic!("no type with id `{:?}`", id),
        }
    }

    fn split_projection<'p>(
        &self,
        projection: &'p ProjectionTy<ChalkIr>,
    ) -> (
        Arc<AssociatedTyDatum>,
        &'p [Parameter<ChalkIr>],
        &'p [Parameter<ChalkIr>],
    ) {
        self.program_ir().unwrap().split_projection(projection)
    }
}
