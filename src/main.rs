mod config;
mod execution_service;
mod sequencer_client;
use tracing::info;

mod chess;
use color_eyre::eyre;
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let mut cfg = config::Config::default();
    println!("config: {:?}", cfg);
    chess::chess::run_until_stopped(cfg).await?;
    Ok(())
}
