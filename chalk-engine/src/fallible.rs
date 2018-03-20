/// Many of our internal operations (e.g., unification) are an attempt
/// to perform some operation which may not complete.
pub type Fallible<T> = Result<T, NoSolution>;

/// Indicates that the attempted operation has "no solution" -- i.e.,
/// cannot be performed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoSolution;
