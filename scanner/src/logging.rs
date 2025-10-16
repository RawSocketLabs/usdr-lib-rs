// Metrea LLC Intellectual Property
// Originally developed by Raw Socket Labs LLC

use std::path::Path;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use tracing_appender::rolling::daily;
use crate::cli::Cli;

/// Initialize the tracing subscriber with multiple output layers
pub fn init_logging(args: &Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create log directory if it doesn't exist
    let log_dir = Path::new(&args.log_dir);
    std::fs::create_dir_all(log_dir)?;

    // Create file appenders for rolling logs
    let file_appender = daily(&args.log_dir, "sdrscanner.log");
    let json_appender = daily(&args.log_dir, "sdrscanner.json");

    // Create environment filter based on CLI argument
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(&format!("sdrscanner={}", args.log_level))
        });

    // Console layer - human-readable output to stderr
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .with_filter(env_filter.clone());

    let registry = tracing_subscriber::registry();

    if args.log_to_file {
        // File layer - human-readable output to rolling files
        let file_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_writer(file_appender)
            .with_filter(env_filter.clone());

        // JSON layer - structured JSON output to rolling files
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_writer(json_appender)
            .with_filter(env_filter.clone());

        registry
            .with(console_layer)
            .with(file_layer)
            .with(json_layer)
            .init();
    } else {
        // Only console output if file logging is disabled
        registry
            .with(console_layer)
            .init();
    }

    tracing::info!("Logging initialized with level: {}", args.log_level);
    if args.log_to_file {
        tracing::info!("Log files will be written to: {}", args.log_dir);
    }

    Ok(())
}
