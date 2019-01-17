use crate::query;

#[derive(Default)]
pub struct ChalkDatabase {
    runtime: salsa::Runtime<ChalkDatabase>,
}

impl salsa::Database for ChalkDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime<ChalkDatabase> {
        &self.runtime
    }
}

salsa::database_storage! {
    pub struct DatabaseStorage for ChalkDatabase {
        impl query::LoweringDatabase {
            fn program_text() for query::ProgramText;
            fn lowered_program() for query::LoweredProgram;
        }
    }
}
