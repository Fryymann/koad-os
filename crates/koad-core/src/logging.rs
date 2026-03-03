use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use tracing_appender::non_blocking::WorkerGuard;
use std::path::PathBuf;

pub fn init_logging(service_name: &str, log_dir: Option<PathBuf>) -> Option<WorkerGuard> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true);

    if let Some(path) = log_dir {
        let file_appender = tracing_appender::rolling::daily(path, format!("{}.log", service_name));
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        
        let file_layer = fmt::layer()
            .with_ansi(false)
            .with_target(true)
            .with_writer(non_blocking)
            .json();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
            
        Some(guard)
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .init();
        None
    }
}
