mod instantiate;
mod quantify;
mod table;
mod var;
mod lifetime_var;

pub use self::table::InferenceTable;
pub use self::table::InferenceSnapshot;
pub use self::table::ParameterInferenceVariable;
pub use self::table::UnificationResult;
pub use self::var::InferenceVariable;
pub use self::lifetime_var::LifetimeInferenceVariable;
