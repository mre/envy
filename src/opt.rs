use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "envy", about = "context-based environment variables")]
pub struct Envy {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Export environment variables based on the current directory
    Export { shell: String },
    /// Print the hook to activate envy for your shell
    Hook { shell: String },
    /// Edit the envy config file
    Edit,
    /// Show envy config for current directory
    Show,
    /// Find a single environment variable and print its value
    Find {
        #[arg(name = "VARIABLE")]
        variable: String,
    },
    /// Print path to envy config file
    Path,
    /// Load environment variables from a given `.env` file (for the current session only)
    Load {
        #[arg(value_parser, default_value = ".env")]
        env_file: PathBuf,
    },
    /// Grants envy to load the given `.env` file
    Allow {
        #[arg(value_parser, default_value = ".env")]
        env_file: PathBuf,
    },
    /// Revokes the authorization of a given `.env` file
    Deny {
        #[arg(value_parser, default_value = ".env")]
        env_file: PathBuf,
    },
}
