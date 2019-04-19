use chalk_solve::wf::WfError;
use failure::Error;

/// Wrapper type for the various errors that can occur during chalk
/// processing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChalkError {
    /// For now, we just convert the error into a string, which makes
    /// it trivially hashable etc.
    error_text: String,
}

impl From<Error> for ChalkError {
    fn from(value: Error) -> Self {
        ChalkError {
            error_text: value.to_string(),
        }
    }
}

impl From<WfError> for ChalkError {
    fn from(value: WfError) -> Self {
        ChalkError {
            error_text: value.to_string(),
        }
    }
}

impl std::fmt::Display for ChalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_text)
    }
}

impl std::error::Error for ChalkError {}
