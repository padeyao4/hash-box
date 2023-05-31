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
    },

    Delete {
        /// package name
        name: String,
    },

    Get {
        /// name
        name: String,
    },

    List {},

    About {},

    Sync {
        /// the path of the file
        path: PathBuf,
    },

    Clear {},
}
