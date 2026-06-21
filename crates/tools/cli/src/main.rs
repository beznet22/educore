//! `educore-cli` — sample binary demonstrating daily engine operations.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use clap::Parser;
use educore_cli::{dispatch, Cli};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match dispatch(cli.command).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e:?}");
            std::process::ExitCode::FAILURE
        }
    }
}
