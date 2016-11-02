mod leaf;
mod table;
mod universe;
mod var;

pub use self::table::InferenceTable;
pub use self::universe::UniverseIndex;
pub use self::var::InferenceVariable;

#[cfg(test)]
mod test;
