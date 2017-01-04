mod instantiate;
mod quantify;
mod table;
mod universe;
mod var;

pub use self::quantify::Quantified;
pub use self::table::InferenceTable;
pub use self::table::InferenceSnapshot;
pub use self::universe::UniverseIndex;
pub use self::var::InferenceVariable;
