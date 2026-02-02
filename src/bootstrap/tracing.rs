pub fn init_tracing() {
  use tracing_subscriber::{fmt, prelude::*, EnvFilter};

  let fmt_layer = fmt::layer()
    .with_target(false)
    .with_thread_ids(false)
    .with_thread_names(false)
    .with_line_number(true)
    .with_file(true);

  let filter_layer = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| "warn,cayopay_server=debug,tower_http=debug".into());

  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();
}
