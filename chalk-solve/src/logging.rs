use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Run an action with a tracing log subscriber. The logging level is loaded
/// from `CHALK_DEBUG`.
pub fn with_tracing_logs<T>(action: impl FnOnce() -> T) -> T {
    let filter = EnvFilter::from_env("CHALK_DEBUG");
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_ansi(false)
        .without_time()
        .finish();
    tracing::subscriber::with_default(subscriber, action)
}
