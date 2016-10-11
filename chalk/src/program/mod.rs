//! Interpreter state.

pub struct Program<C> {
    obligations: Vec<Obligation<C>>
}

mod obligation;
pub use self::obligation::Obligation;

mod environment;
pub use self::environment::Environment;
