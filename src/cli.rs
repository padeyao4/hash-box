use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        /// the path of the file
        path: PathBuf,
        // #[arg(short, long)]
        // force: bool,
    },

    Delete {
        /// package name
        name: String,
    },

    Get {
        /// name
        name: String,
        /// the path to save files
        path: Option<PathBuf>,
    },

    List {},

    About {},

    Sync {
        /// the path of the file
        path: PathBuf,
    },

    Clear {},
}
