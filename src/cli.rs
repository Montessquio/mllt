use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mllt")]
#[command(about = "A tiny static site generator designed for self-hosting linktree-like pages.", long_about = None)]
pub struct Cli {
    /// Sets the verbosity level.
    #[arg(short, long, global = true, action = clap::ArgAction::Count, conflicts_with = "quiet")]
    pub verbose: u8,

    /// Suppress all messages besides errors.
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Render the site to static HTML.
    #[command(alias = "b")]
    Build {
        /// Enable strict mode in the handlebars parser. This causes
        /// missing or unknown values to produce hard errors instead of
        /// empty strings.
        #[arg(long, action = clap::ArgAction::SetTrue)]
        strict: Option<bool>,

        /// Overrides the output folder path specified in the config file.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overrides the content folder path specified in the config file.
        #[arg(long)]
        content: Option<PathBuf>,

        /// Overrides the theme folder path specified in the config file.
        #[arg(long)]
        theme: Option<PathBuf>,

        /// Overrides the assets folder path specified in the config file.
        #[arg(long)]
        assets: Option<PathBuf>,

        /// Path to the config file.
        #[arg(short, long, default_value = "./mllt.toml")]
        config: PathBuf,
    },

    /// Start a local development server.
    #[command(alias = "s")]
    Serve {
        /// Port to bind the server to.
        #[arg(short, long, default_value = "1313")]
        port: u16,

        /// Enable strict mode in the handlebars parser. This causes
        /// missing or unknown values to produce hard errors instead of
        /// empty strings.
        #[arg(long, action = clap::ArgAction::SetTrue)]
        strict: Option<bool>,

        /// Overrides the content folder path specified in the config file.
        #[arg(long)]
        content: Option<PathBuf>,

        /// Overrides the theme folder path specified in the config file.
        #[arg(long)]
        theme: Option<PathBuf>,

        /// Overrides the assets folder path specified in the config file.
        #[arg(long)]
        assets: Option<PathBuf>,

        /// Path to the config file.
        #[arg(short, long, default_value = "./mllt.toml")]
        config: PathBuf,
    },

    /// Create a new mllt site at the given path.
    #[command(alias = "n")]
    New {
        /// Create the new project even if the destination is
        /// non-empty, overwriting files if needed.
        #[arg(long)]
        force: bool,

        /// The name of the project, which is the path to the
        /// project root.
        #[arg()]
        base_path: PathBuf
    },
}
