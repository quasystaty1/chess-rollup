mod config;
mod execution_service;
mod rollup_app;
mod sequencer_client;

mod chess;
use color_eyre::eyre;
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cfg = config::Config::default();
    println!("config: {:?}", cfg);
    chess::Chess::run_until_stopped(cfg).await?;
    Ok(())
}
