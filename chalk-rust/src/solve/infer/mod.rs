mod table;
mod universe;
mod var;

pub use self::table::InferenceTable;
pub use self::table::InferenceSnapshot;
pub use self::table::UnifyError;
pub use self::table::UnifyResult;
pub use self::universe::UniverseIndex;
pub use self::var::InferenceVariable;
