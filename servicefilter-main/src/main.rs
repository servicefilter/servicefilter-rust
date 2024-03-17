mod cmd;
mod error;
mod app;
mod service;

use crate::cmd::cli::Cli;

use structopt::StructOpt;
use servicefilter_core::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "servicefilter=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::from_args();
    let check_result = cli.check().await;
    if let Err(e) = check_result {
        tracing::error!("cli check error: {:?}", e);
        return Err(e);
    }

    let start_result = app::start(cli, tokio::signal::ctrl_c()).await;
    if let Err(e) = start_result {
        tracing::error!("start error: {:?}", e);
        return Err(e);
    }

    return Ok(());
}
