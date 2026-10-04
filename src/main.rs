use clap::{Parser, Subcommand};

mod config;
mod player;
mod tui;
mod youtube;
mod utils;

#[derive(Parser)]
#[command(name = "sounding", version, about = "A small music player written in Rust (no affiliation w the sexual act)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Main {},
    Config {},
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };

    match cli.command.unwrap_or(Commands::Main {}) {

        Commands::Main {} => {
                let config = config::Config::load()?;
                let mut app = tui::App::new(config)?;

                ratatui::run(|terminal| {
                    app.run(terminal)
                })?;
            }

        Commands::Config {} => {
            let _ = config::Config::load();
        }
    }
    Ok(())
}
