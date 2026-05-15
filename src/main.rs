use clap::{Parser, Subcommand};

mod config;
mod proxy;
mod runner;

#[derive(Parser)]
#[command(name = "lcc", about = "Launch Claude Code with custom profiles")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch claude with a profile
    Start {
        /// Profile name from settings.json
        #[arg(long)]
        profil: String,
        /// Extra arguments passed to claude
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// List available profiles
    List,
    /// Open or validate settings
    Settings {
        /// Validate the settings.json structure
        #[arg(long)]
        validate: bool,
    },
    /// Gestion du proxy LiteLLM autonome.
    Proxy {
        #[command(subcommand)]
        cmd: proxy::cli::Proxy,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start { profil, args } => {
            if let Err(e) = runner::run(&profil, &args) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::List => {
            if let Err(e) = config::list_profiles() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Settings { validate } => {
            if let Err(e) = config::settings_command(validate) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Commands::Proxy { cmd } => {
            if let Err(e) = proxy::cli::dispatch(cmd) {
                eprintln!("✗ {e}");
                std::process::exit(1);
            }
        }
    }
}
