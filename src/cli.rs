use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::{server, settings::Settings};

/// A dumb web service for controlling certain govee light strips.
#[derive(Debug, Clone, Parser)]
#[command(version, max_term_width = 120)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Self::parse();

        cli.command.run().await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Check(Check),
    Server(Server),
}

impl Command {
    async fn run(&self) -> Result<()> {
        match self {
            Command::Check(cmd) => cmd.run().await,
            Command::Server(cmd) => cmd.run().await,
        }
    }
}

/// Check that the settings are valid (env and config file)
#[derive(Debug, Clone, Args)]
pub struct Check {}

impl Check {
    async fn run(&self) -> Result<()> {
        let settings = Settings::new()?;

        println!("Settings OK\n\n{}", &settings);

        Ok(())
    }
}

/// Start the govee-web server.
#[derive(Debug, Clone, Args)]
pub struct Server {}

impl Server {
    async fn run(&self) -> Result<()> {
        server::serve(Settings::new()?).await
    }
}
