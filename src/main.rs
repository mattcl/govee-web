use anyhow::Result;

pub mod cli;
pub mod controllers;
pub mod error;
pub mod repositories;
pub mod routes;
pub mod server;
pub mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    cli::Cli::run().await
}
