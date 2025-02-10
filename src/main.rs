use std::time::Instant;
use clap::Parser;
use cli::{Cli, Command};
use config::Config;
use site::Site;
use color_eyre::eyre::Result;
use tracing::{debug, info};

mod cli;
mod config;
mod site;

fn main() -> Result<()> {
    let cli: cli::Cli = Cli::parse();
    init_tracing(cli.verbose);

    debug!("Strike the Earth!");
    
    // Some CLI flags overwrite config file options.
    // merge_with applies this into one, single config struct.
    let config = Config::from_file(cli.config.as_path())?.merge_with(&cli);
    debug!("Final Config: {config:#?}");

    match &cli.command {
        Some(Command::Serve { port }) => {
            render(&config)?;
            tracing::info!("Starting development server on port {}", port);
            // Add server logic here
            todo!()
        }
        None => {
            render(&config)
        }
    }
}

fn render(config: &Config) -> Result<()> {
    let now = Instant::now();
    info!("Building site to \"{}\"", config.out_dir.display());
    let site = Site::new(config)?;
    site.render()?;
    info!("Done! Took {}", format_duration(now.elapsed())?);
    Ok(())
}

fn init_tracing(verbosity: u8) {
    match verbosity {
        0 => tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init(),
        1 => tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init(),
        _ => tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init(),
    }
}

fn format_duration(duration: std::time::Duration) -> Result<String> {
    let duration = chrono::Duration::from_std(duration)?;

    // Extract hours, minutes, seconds, and milliseconds
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    let milliseconds = duration.num_milliseconds() % 1000;

    // Format based on the duration length
    let out = if duration.num_seconds() < 1 {
        // Less than 1 second: display in milliseconds
        format!("{}ms", milliseconds)
    } else if duration.num_seconds() < 60 {
        // Less than 1 minute: display in seconds and milliseconds
        format!("{}.{:03}s", seconds, milliseconds)
    } else if duration.num_seconds() < 3600 {
        // Less than 1 hour: display in minutes, seconds, and milliseconds
        format!("{}m {:02}.{:03}s", minutes, seconds, milliseconds)
    } else {
        // 1 hour or more: display in hours, minutes, seconds, and milliseconds
        format!(
            "{}h {:02}m {:02}.{:03}s",
            hours, minutes, seconds, milliseconds
        )
    };

    Ok(out)
}