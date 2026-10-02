use clap::{Parser, Subcommand};

mod config;
mod player;
mod tui;
mod youtube;

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
    /// Greet someone
    Hello {
        /// Name to greet
        name: String,
    },

    /// Add two numbers
    Add {
        a: i32,
        b: i32,
    },

    Gau {

    },

    Config {

    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };

    match cli.command.unwrap_or(Commands::Gau {}) {
        Commands::Hello { name } => {
            println!("Hello, {name}!");
        }
        Commands::Add { a, b } => {
            println!("{}", a + b);
        }
        Commands::Gau {} => {
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

