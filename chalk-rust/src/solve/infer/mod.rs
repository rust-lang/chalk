mod instantiate;
mod lifetime_var;
mod quantify;
mod table;
mod unify;
mod var;
#[cfg(test)] mod test;

pub use self::table::InferenceTable;
pub use self::table::InferenceSnapshot;
pub use self::table::ParameterInferenceVariable;
pub use self::unify::UnificationResult;
pub use self::var::InferenceVariable;
pub use self::lifetime_var::LifetimeInferenceVariable;
