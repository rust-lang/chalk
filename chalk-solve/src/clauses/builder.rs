use crate::cast::{Cast, Caster};
use crate::RustIrDatabase;
use chalk_ir::family::{ChalkIr, HasTypeFamily};
use chalk_ir::fold::Fold;
use chalk_ir::*;
use chalk_rust_ir::*;

pub struct ClauseBuilder<'me> {
    pub db: &'me dyn RustIrDatabase,
    clauses: &'me mut Vec<ProgramClause<ChalkIr>>,
    binders: Vec<ParameterKind<()>>,
    parameters: Vec<Parameter<ChalkIr>>,
}

impl<'me> ClauseBuilder<'me> {
    pub fn new(db: &'me dyn RustIrDatabase, clauses: &'me mut Vec<ProgramClause<ChalkIr>>) -> Self {
        Self {
            db,
            clauses,
            binders: vec![],
            parameters: vec![],
        }
    }

    pub fn push_fact(&mut self, consequence: impl Cast<DomainGoal<ChalkIr>>) {
        self.push_clause(consequence, None::<Goal<_>>);
    }

    pub fn push_clause(
        &mut self,
        consequence: impl Cast<DomainGoal<ChalkIr>>,
        conditions: impl IntoIterator<Item = impl Cast<Goal<ChalkIr>>>,
    ) {
        let clause = ProgramClauseImplication {
            consequence: consequence.cast(),
            conditions: conditions.into_iter().casted().collect(),
        };

        if self.binders.len() == 0 {
            self.clauses.push(ProgramClause::Implies(clause));
        } else {
            self.clauses.push(ProgramClause::ForAll(Binders {
                binders: self.binders.clone(),
                value: clause,
            }));
        }

        debug!("pushed clause {:?}", self.clauses.last());
    }

    /// Accesses the placeholders for the current list of parameters in scope.
    pub fn placeholders_in_scope(&self) -> &[Parameter<ChalkIr>] {
        &self.parameters
    }

    /// Executes `op` with the `binders` in-scope; `op` is invoked
    /// with the bound value `v` as a parameter. After `op` finishes,
    /// the binders are popped from scope.
    ///
    /// The new binders are always pushed onto the end of the internal
    /// list of binders; this means that any extant values where were
    /// created referencing the *old* list of binders are still valid.
    pub fn push_binders<V>(&mut self, binders: &Binders<V>, op: impl FnOnce(&mut Self, V::Result))
    where
        V: Fold<ChalkIr> + HasTypeFamily<TypeFamily = ChalkIr>,
    {
        let old_len = self.binders.len();
        self.binders.extend(binders.binders.clone());
        self.parameters.extend(
            binders
                .binders
                .iter()
                .zip(old_len..)
                .map(|p| p.to_parameter()),
        );

        let value = binders.substitute(&self.parameters[old_len..]);
        op(self, value);

        self.binders.truncate(old_len);
        self.parameters.truncate(old_len);
    }
}
