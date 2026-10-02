use clap::{Parser, Subcommand};
use std::path::Path;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};

mod config;
mod player;
mod tui;
mod youtube;

const SOCKET: &str = "/tmp/sounding.sock";

#[derive(Parser)]
#[command(name = "sounding", version, about = "A small music player written in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, help = "Sets a custom file path")]
    file: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Hello {
        name: String,
    },

    Add {
        a: i32,
        b: i32,
    },

    Main {},

    Config {},
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };

    match cli.command.unwrap_or(Commands::Main {}) {
        Commands::Hello { name } => {
            println!("Hello, {name}!");
        }

        Commands::Add { a, b } => {
            println!("{}", a + b);
        }

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
