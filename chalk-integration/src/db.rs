use crate::error::ChalkError;
use crate::lowering::LowerGoal;
use crate::program::Program;
use crate::query::{Lowering, LoweringDatabase};
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
use chalk_ir::StructId;
use chalk_ir::TraitId;
use chalk_ir::TypeId;
use chalk_ir::TypeKindId;
use chalk_ir::UCanonical;
use chalk_rust_ir::AssociatedTyDatum;
use chalk_rust_ir::AssociatedTyValue;
use chalk_rust_ir::AssociatedTyValueId;
use chalk_rust_ir::ImplDatum;
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

    pub fn solve(
        &self,
        goal: &UCanonical<InEnvironment<Goal<ChalkIr>>>,
    ) -> Option<Solution<ChalkIr>> {
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

impl RustIrDatabase<ChalkIr> for ChalkDatabase {
    fn custom_clauses(&self) -> Vec<ProgramClause<ChalkIr>> {
        self.program_ir().unwrap().custom_clauses()
    }

    fn associated_ty_data(&self, ty: TypeId) -> Arc<AssociatedTyDatum<ChalkIr>> {
        self.program_ir().unwrap().associated_ty_data(ty)
    }

    fn trait_datum(&self, id: TraitId) -> Arc<TraitDatum<ChalkIr>> {
        self.program_ir().unwrap().trait_datum(id)
    }

    fn impl_datum(&self, id: ImplId) -> Arc<ImplDatum<ChalkIr>> {
        self.program_ir().unwrap().impl_datum(id)
    }

    fn associated_ty_value(&self, id: AssociatedTyValueId) -> Arc<AssociatedTyValue<ChalkIr>> {
        self.program_ir().unwrap().associated_ty_values[&id].clone()
    }

    fn struct_datum(&self, id: StructId) -> Arc<StructDatum<ChalkIr>> {
        self.program_ir().unwrap().struct_datum(id)
    }

    fn impls_for_trait(&self, trait_id: TraitId, parameters: &[Parameter<ChalkIr>]) -> Vec<ImplId> {
        self.program_ir()
            .unwrap()
            .impls_for_trait(trait_id, parameters)
    }

    fn local_impls_to_coherence_check(&self, trait_id: TraitId) -> Vec<ImplId> {
        self.program_ir()
            .unwrap()
            .local_impls_to_coherence_check(trait_id)
    }

    fn impl_provided_for(&self, auto_trait_id: TraitId, struct_id: StructId) -> bool {
        self.program_ir()
            .unwrap()
            .impl_provided_for(auto_trait_id, struct_id)
    }

    fn type_name(&self, id: TypeKindId) -> Identifier {
        self.program_ir().unwrap().type_name(id)
    }
}
