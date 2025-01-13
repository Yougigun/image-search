use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_line_number(true);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug,hyper=off".into()))
        .with(fmt_layer)
        .init();
}
