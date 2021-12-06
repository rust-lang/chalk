/// Run an action with a tracing log subscriber. The logging level is loaded
/// from `CHALK_DEBUG`.
#[cfg(feature = "tracing-full")]
pub fn with_tracing_logs<T>(action: impl FnOnce() -> T) -> T {
    use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
    use tracing_tree::HierarchicalLayer;
    let filter = EnvFilter::from_env("CHALK_DEBUG");
    let subscriber = Registry::default()
        .with(filter)
        .with(HierarchicalLayer::new(2).with_writer(std::io::stdout));
    tracing::subscriber::with_default(subscriber, action)
}

/// Run an action with a tracing log subscriber. The logging level is loaded
/// from `CHALK_DEBUG`.
#[cfg(not(feature = "tracing-full"))]
pub fn with_tracing_logs<T>(action: impl FnOnce() -> T) -> T {
    action()
}
