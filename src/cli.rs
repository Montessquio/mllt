use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mllt")]
#[command(about = "A tiny static site generator designed for self-hosting linktree-like pages.", long_about = None)]
pub struct Cli {
    /// Sets the verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Overrides the output folder specified in the config file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overrides the theme folder specified in the config file
    #[arg(short, long)]
    pub theme: Option<PathBuf>,

    /// Overrides the assets folder specified in the config file
    #[arg(short, long)]
    pub assets: Option<PathBuf>,

    /// Path to the config file
    #[arg(short, long, default_value = "mllt.toml")]
    pub config: PathBuf,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start a local development server
    Serve {
        /// Port to bind the server to
        #[arg(short, long, default_value = "1313")]
        port: u16,
    },
}