//! Select `FxHashMap` and friends either from compiler or from
//! `fxhash` depending.

#[cfg(feature = "stabler")]
pub use fxhash::{FxHashMap, FxHashSet};

#[cfg(not(feature = "stabler"))]
pub use rustc_data_structures::fx::{FxHashMap, FxHashSet};
