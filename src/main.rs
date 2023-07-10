use anyhow::Result;

pub mod cli;
pub mod controllers;
pub mod error;
pub mod routes;
pub mod server;
pub mod settings;
pub mod store;

#[tokio::main]
async fn main() -> Result<()> {
    cli::Cli::run().await
}
